use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::{Json, Router, routing::get};
use serde::Deserialize;

use forge::AppContext;

use crate::config::MoviesConfig;
use crate::tmdb::{MediaItem, MediaType, SearchResult, TmdbClient};

pub fn router() -> Router<AppContext> {
    Router::new()
        .route("/search", get(search))
        .route("/movie/{id}", get(movie_details))
        .route("/tv/{id}", get(tv_details))
        .route("/image/{*path}", get(image_proxy))
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
}

async fn search(
    State(ctx): State<AppContext>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let config = ctx.config.module_config::<MoviesConfig>("movies")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let results = tmdb.search(&params.q).await?;
    Ok(Json(results))
}

async fn movie_details(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Json<MediaItem>, AppError> {
    let config = ctx.config.module_config::<MoviesConfig>("movies")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Movie, id).await?;
    Ok(Json(item))
}

async fn tv_details(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Json<MediaItem>, AppError> {
    let config = ctx.config.module_config::<MoviesConfig>("movies")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Tv, id).await?;
    Ok(Json(item))
}

async fn image_proxy(
    State(ctx): State<AppContext>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let path = if path.starts_with("w") || path.starts_with("original") {
        path
    } else {
        format!("original/{path}")
    };
    let url = format!("https://image.tmdb.org/t/p/{path}");
    let res = ctx
        .http
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| forge::Error::Generic(e.to_string()))?;

    let content_type = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    let bytes = res.bytes().await?;

    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "public, max-age=86400".into()),
        ],
        bytes,
    ))
}

struct AppError(forge::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl<E: Into<forge::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(e.into())
    }
}
