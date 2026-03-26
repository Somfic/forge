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

use crate::config::CinemaConfig;
use crate::streams::Stream;
use crate::subtitles::{SubtitleCue, SubtitleTrack};
use crate::tmdb::{MediaItem, MediaType, SearchResult, TmdbClient};
use crate::torrentio::StremioClient;

pub fn router() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(search))
        .routes(routes!(movie_details))
        .routes(routes!(tv_details))
        .routes(routes!(movie_streams))
        .routes(routes!(tv_streams))
        .routes(routes!(start_stream))
        .routes(routes!(movie_subtitles))
        .routes(routes!(tv_subtitles))
        .routes(routes!(subtitle_cues))
        .routes(routes!(trending))
        .routes(routes!(similar))
        .routes(routes!(record_watch))
        .routes(routes!(watch_history))
        .routes(routes!(add_to_collection))
        .routes(routes!(remove_from_collection))
        .routes(routes!(get_collection))
        .routes(routes!(is_in_collection))
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
    let config = ctx.config.module_config::<CinemaConfig>("cinema")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let results = tmdb.search(&params.q).await?;
    Ok(Json(results))
}

#[utoipa::path(get, path = "/movie/{id}", responses((status = 200, body = MediaItem)))]
async fn movie_details(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Json<MediaItem>, AppError> {
    let config = ctx.config.module_config::<CinemaConfig>("cinema")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Movie, id).await?;
    Ok(Json(item))
}

#[utoipa::path(get, path = "/tv/{id}", responses((status = 200, body = MediaItem)))]
async fn tv_details(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Json<MediaItem>, AppError> {
    let config = ctx.config.module_config::<CinemaConfig>("cinema")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Tv, id).await?;
    Ok(Json(item))
}

#[utoipa::path(get, path = "/streams/movie/{id}", responses((status = 200, body = Vec<Stream>)))]
async fn movie_streams(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<Stream>>, AppError> {
    let config = ctx.config.module_config::<CinemaConfig>("cinema")?;

    // Get IMDB ID from TMDB
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Movie, id).await?;
    let imdb_id = item
        .imdb_id
        .ok_or_else(|| forge::Error::Generic("No IMDB ID found for this movie".into()))?;

    let path = format!("movie/{}", imdb_id);
    let streams = crate::streams::aggregate(&ctx.http, &config.stream_sources, &path).await;

    Ok(Json(streams))
}

#[utoipa::path(get, path = "/streams/tv/{id}/{season}/{episode}", responses((status = 200, body = Vec<Stream>)))]
async fn tv_streams(
    State(ctx): State<AppContext>,
    Path((id, season, episode)): Path<(i64, i64, i64)>,
) -> Result<Json<Vec<Stream>>, AppError> {
    let config = ctx.config.module_config::<CinemaConfig>("cinema")?;

    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Tv, id).await?;
    let imdb_id = item
        .imdb_id
        .ok_or_else(|| forge::Error::Generic("No IMDB ID found for this show".into()))?;

    let path = format!("series/{}:{}:{}", imdb_id, season, episode);
    let streams = crate::streams::aggregate(&ctx.http, &config.stream_sources, &path).await;

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
    let config = ctx.config.module_config::<CinemaConfig>("cinema")?;
    let torrentio = StremioClient::new(ctx.http.clone(), config.stremio_url.clone());
    let url = torrentio.start(&info_hash, file_idx).await?;
    Ok(Json(StartStreamResponse { url }))
}

#[utoipa::path(get, path = "/subtitles/movie/{id}", responses((status = 200, body = Vec<SubtitleTrack>)))]
async fn movie_subtitles(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<SubtitleTrack>>, AppError> {
    let config = ctx.config.module_config::<CinemaConfig>("cinema")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Movie, id).await?;
    let imdb_id = item
        .imdb_id
        .ok_or_else(|| forge::Error::Generic("No IMDB ID found".into()))?;

    let path = format!("movie/{}", imdb_id);
    let tracks = crate::subtitles::fetch_tracks(&ctx.http, &path, &config.subtitle_languages).await;
    Ok(Json(tracks))
}

#[utoipa::path(get, path = "/subtitles/tv/{id}/{season}/{episode}", responses((status = 200, body = Vec<SubtitleTrack>)))]
async fn tv_subtitles(
    State(ctx): State<AppContext>,
    Path((id, season, episode)): Path<(i64, i64, i64)>,
) -> Result<Json<Vec<SubtitleTrack>>, AppError> {
    let config = ctx.config.module_config::<CinemaConfig>("cinema")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let item = tmdb.details(MediaType::Tv, id).await?;
    let imdb_id = item
        .imdb_id
        .ok_or_else(|| forge::Error::Generic("No IMDB ID found".into()))?;

    let path = format!("series/{}:{}:{}", imdb_id, season, episode);
    let tracks = crate::subtitles::fetch_tracks(&ctx.http, &path, &config.subtitle_languages).await;
    Ok(Json(tracks))
}

#[derive(Deserialize, IntoParams)]
struct SubtitleCueParams {
    /// URL of the SRT subtitle file
    url: String,
}

#[utoipa::path(get, path = "/subtitles/cues",
    params(SubtitleCueParams),
    responses((status = 200, body = Vec<SubtitleCue>))
)]
async fn subtitle_cues(
    State(ctx): State<AppContext>,
    Query(params): Query<SubtitleCueParams>,
) -> Result<Json<Vec<SubtitleCue>>, AppError> {
    let cues = crate::subtitles::fetch_cues(&ctx.http, &params.url).await;
    Ok(Json(cues))
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

// ── Browse endpoints ──

#[utoipa::path(get, path = "/trending", responses((status = 200, body = Vec<SearchResult>)))]
async fn trending(State(ctx): State<AppContext>) -> Result<Json<Vec<SearchResult>>, AppError> {
    let config = ctx.config.module_config::<CinemaConfig>("cinema")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let results = tmdb.trending().await?;
    Ok(Json(results))
}

#[utoipa::path(get, path = "/similar/{type}/{id}", responses((status = 200, body = Vec<SearchResult>)))]
async fn similar(
    State(ctx): State<AppContext>,
    Path((media_type, id)): Path<(String, i64)>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let config = ctx.config.module_config::<CinemaConfig>("cinema")?;
    let tmdb = TmdbClient::new(&config, ctx.http.clone());
    let mt = match media_type.as_str() {
        "movie" => MediaType::Movie,
        "tv" => MediaType::Tv,
        _ => return Err(forge::Error::Generic("Invalid media type".into()).into()),
    };
    let results = tmdb.similar(mt, id).await?;
    Ok(Json(results))
}

// ── Watch history ──

#[derive(Deserialize, Serialize, ToSchema)]
struct RecordWatchRequest {
    media_type: String,
    tmdb_id: i64,
    title: String,
    poster_path: Option<String>,
    season: Option<i64>,
    episode: Option<i64>,
    info_hash: Option<String>,
    file_idx: Option<i64>,
    progress: Option<f64>,
    duration: Option<f64>,
}

#[derive(Serialize, ToSchema, sqlx::FromRow)]
struct WatchHistoryItem {
    media_type: String,
    tmdb_id: i64,
    title: String,
    poster_path: Option<String>,
    season: i64,
    episode: i64,
    info_hash: Option<String>,
    file_idx: i64,
    progress: f64,
    duration: f64,
    last_watched: String,
}

#[utoipa::path(post, path = "/watch",
    request_body = RecordWatchRequest,
    responses((status = 204))
)]
async fn record_watch(
    State(ctx): State<AppContext>,
    Json(body): Json<RecordWatchRequest>,
) -> Result<StatusCode, AppError> {
    sqlx::query(
        "INSERT INTO watch_history (media_type, tmdb_id, title, poster_path, season, episode, info_hash, file_idx, progress, duration)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(media_type, tmdb_id)
         DO UPDATE SET title = excluded.title, poster_path = excluded.poster_path, season = excluded.season, episode = excluded.episode, info_hash = excluded.info_hash, file_idx = excluded.file_idx, progress = excluded.progress, duration = excluded.duration, last_watched = datetime('now')"
    )
    .bind(&body.media_type)
    .bind(body.tmdb_id)
    .bind(&body.title)
    .bind(&body.poster_path)
    .bind(body.season.unwrap_or(0))
    .bind(body.episode.unwrap_or(0))
    .bind(&body.info_hash)
    .bind(body.file_idx.unwrap_or(0))
    .bind(body.progress.unwrap_or(0.0))
    .bind(body.duration.unwrap_or(0.0))
    .execute(&ctx.db)
    .await
    .map_err(|e| forge::Error::Generic(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/watch/history", responses((status = 200, body = Vec<WatchHistoryItem>)))]
async fn watch_history(
    State(ctx): State<AppContext>,
) -> Result<Json<Vec<WatchHistoryItem>>, AppError> {
    let items = sqlx::query_as::<_, WatchHistoryItem>(
        "SELECT media_type, tmdb_id, title, poster_path, season, episode, info_hash, file_idx, progress, duration, last_watched
         FROM watch_history ORDER BY last_watched DESC LIMIT 20"
    )
    .fetch_all(&ctx.db)
    .await
    .map_err(|e| forge::Error::Generic(e.to_string()))?;

    Ok(Json(items))
}

// ── Collections (watchlist, favorites, etc.) ──

#[derive(Deserialize, Serialize, ToSchema)]
struct CollectionRequest {
    collection: String,
    media_type: String,
    tmdb_id: i64,
    title: String,
    poster_path: Option<String>,
}

#[derive(Serialize, ToSchema, sqlx::FromRow)]
struct CollectionItem {
    collection: String,
    media_type: String,
    tmdb_id: i64,
    title: String,
    poster_path: Option<String>,
    added_at: String,
}

#[derive(Serialize, ToSchema)]
struct CollectionStatus {
    in_collection: bool,
}

#[utoipa::path(post, path = "/collection",
    request_body = CollectionRequest,
    responses((status = 204))
)]
async fn add_to_collection(
    State(ctx): State<AppContext>,
    Json(body): Json<CollectionRequest>,
) -> Result<StatusCode, AppError> {
    sqlx::query(
        "INSERT INTO collections (collection, media_type, tmdb_id, title, poster_path)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(collection, media_type, tmdb_id) DO UPDATE SET title = excluded.title, poster_path = excluded.poster_path"
    )
    .bind(&body.collection)
    .bind(&body.media_type)
    .bind(body.tmdb_id)
    .bind(&body.title)
    .bind(&body.poster_path)
    .execute(&ctx.db)
    .await
    .map_err(|e| forge::Error::Generic(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/collection/{collection}/{type}/{id}", responses((status = 204)))]
async fn remove_from_collection(
    State(ctx): State<AppContext>,
    Path((collection, media_type, id)): Path<(String, String, i64)>,
) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM collections WHERE collection = ? AND media_type = ? AND tmdb_id = ?")
        .bind(&collection)
        .bind(&media_type)
        .bind(id)
        .execute(&ctx.db)
        .await
        .map_err(|e| forge::Error::Generic(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/collection/{collection}", responses((status = 200, body = Vec<CollectionItem>)))]
async fn get_collection(
    State(ctx): State<AppContext>,
    Path(collection): Path<String>,
) -> Result<Json<Vec<CollectionItem>>, AppError> {
    let items = sqlx::query_as::<_, CollectionItem>(
        "SELECT collection, media_type, tmdb_id, title, poster_path, added_at
         FROM collections WHERE collection = ? ORDER BY added_at DESC",
    )
    .bind(&collection)
    .fetch_all(&ctx.db)
    .await
    .map_err(|e| forge::Error::Generic(e.to_string()))?;

    Ok(Json(items))
}

#[utoipa::path(get, path = "/collection/{collection}/{type}/{id}", responses((status = 200, body = CollectionStatus)))]
async fn is_in_collection(
    State(ctx): State<AppContext>,
    Path((collection, media_type, id)): Path<(String, String, i64)>,
) -> Result<Json<CollectionStatus>, AppError> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM collections WHERE collection = ? AND media_type = ? AND tmdb_id = ?",
    )
    .bind(&collection)
    .bind(&media_type)
    .bind(id)
    .fetch_one(&ctx.db)
    .await
    .map_err(|e| forge::Error::Generic(e.to_string()))?;

    Ok(Json(CollectionStatus {
        in_collection: count.0 > 0,
    }))
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
