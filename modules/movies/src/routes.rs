use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use forge::AppContext;

use crate::config::MoviesConfig;
use crate::tmdb::{MediaItem, MediaType, SearchResult, TmdbClient};
use crate::torrentio::{Stream, TorrentioClient};

pub fn router() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(search))
        .routes(routes!(movie_details))
        .routes(routes!(tv_details))
        .routes(routes!(movie_streams))
        .routes(routes!(tv_streams))
        .routes(routes!(start_stream))
        .route("/image/{*path}", axum::routing::get(image_proxy))
}

#[derive(Deserialize, IntoParams)]
struct SearchParams {
    q: String,
}

#[utoipa::path(get, path = "/search",
    params(SearchParams),
    responses((status = 200, body = Vec<SearchResult>))
)]
async fn search(
    State(ctx): State<AppContext>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let config = ctx.config.module_config::<MoviesConfig>("movies")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let results = tmdb.search(&params.q).await?;
    Ok(Json(results))
}

#[utoipa::path(get, path = "/movie/{id}", responses((status = 200, body = MediaItem)))]
async fn movie_details(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Json<MediaItem>, AppError> {
    let config = ctx.config.module_config::<MoviesConfig>("movies")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Movie, id).await?;
    Ok(Json(item))
}

#[utoipa::path(get, path = "/tv/{id}", responses((status = 200, body = MediaItem)))]
async fn tv_details(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Json<MediaItem>, AppError> {
    let config = ctx.config.module_config::<MoviesConfig>("movies")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Tv, id).await?;
    Ok(Json(item))
}

#[utoipa::path(get, path = "/streams/movie/{id}", responses((status = 200, body = Vec<Stream>)))]
async fn movie_streams(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<Stream>>, AppError> {
    let config = ctx.config.module_config::<MoviesConfig>("movies")?;

    // Get IMDB ID from TMDB
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Movie, id).await?;
    let imdb_id = item
        .imdb_id
        .ok_or_else(|| forge::Error::Generic("No IMDB ID found for this movie".into()))?;

    // Query Torrentio for streams
    let torrentio = TorrentioClient::new(ctx.http.clone(), config.stremio_url.clone());
    let streams = torrentio.streams_for_movie(&imdb_id).await?;

    Ok(Json(streams))
}

#[utoipa::path(get, path = "/streams/tv/{id}/{season}/{episode}", responses((status = 200, body = Vec<Stream>)))]
async fn tv_streams(
    State(ctx): State<AppContext>,
    Path((id, season, episode)): Path<(i64, i64, i64)>,
) -> Result<Json<Vec<Stream>>, AppError> {
    let config = ctx.config.module_config::<MoviesConfig>("movies")?;

    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Tv, id).await?;
    let imdb_id = item
        .imdb_id
        .ok_or_else(|| forge::Error::Generic("No IMDB ID found for this show".into()))?;

    let torrentio = TorrentioClient::new(ctx.http.clone(), config.stremio_url.clone());
    let streams = torrentio.streams_for_episode(&imdb_id, season, episode).await?;

    Ok(Json(streams))
}

#[derive(Serialize, ToSchema)]
struct StartStreamResponse {
    url: String,
}

#[utoipa::path(post, path = "/streams/start/{info_hash}/{file_idx}",
    responses((status = 200, body = StartStreamResponse))
)]
async fn start_stream(
    State(ctx): State<AppContext>,
    Path((info_hash, file_idx)): Path<(String, i64)>,
) -> Result<Json<StartStreamResponse>, AppError> {
    let config = ctx.config.module_config::<MoviesConfig>("movies")?;
    let torrentio = TorrentioClient::new(ctx.http.clone(), config.stremio_url.clone());
    let url = torrentio.start(&info_hash, file_idx).await?;
    Ok(Json(StartStreamResponse { url }))
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
