use crate::config::MoviesConfig;
use forge::{HttpClient, json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Clone, ToSchema)]
pub struct MediaItem {
    pub id: i64,
    pub media_type: MediaType,
    pub title: String,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub release_date: Option<String>,
    pub runtime: Option<i64>,
    pub rating: Option<f64>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub genres: Vec<Genre>,
    pub videos: Vec<Video>,
    pub images: Option<Images>,
    pub seasons: Option<Vec<Season>>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Movie,
    Tv,
}

#[derive(Serialize, Clone, ToSchema)]
pub struct SearchResult {
    pub id: i64,
    pub media_type: MediaType,
    pub title: String,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct Genre {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Clone, ToSchema)]
pub struct Video {
    pub key: String,
    pub site: String,
    pub name: String,
    pub video_type: String,
}

#[derive(Serialize, Clone, ToSchema)]
pub struct Images {
    pub posters: Vec<Image>,
    pub backdrops: Vec<Image>,
    pub logos: Vec<Image>,
}

#[derive(Serialize, Clone, ToSchema)]
pub struct Image {
    pub file_path: String,
    pub width: i64,
    pub height: i64,
    pub iso_639_1: Option<String>,
    pub vote_average: f64,
}

#[derive(Serialize, Clone, ToSchema)]
pub struct Season {
    pub id: i64,
    pub season_number: i64,
    pub name: String,
    pub episode_count: i64,
    pub poster_path: Option<String>,
    pub air_date: Option<String>,
}

// --- Raw TMDB response types (private) ---

#[derive(Deserialize)]
struct TmdbMultiSearchResults {
    results: Vec<TmdbMultiSearchResult>,
}

#[derive(Deserialize)]
struct TmdbMultiSearchResult {
    id: i64,
    media_type: String,
    // movie fields
    title: Option<String>,
    release_date: Option<String>,
    // tv fields
    name: Option<String>,
    first_air_date: Option<String>,
    // shared
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
}

impl TmdbMultiSearchResult {
    fn into_search_result(self) -> Option<SearchResult> {
        let media_type = match self.media_type.as_str() {
            "movie" => MediaType::Movie,
            "tv" => MediaType::Tv,
            _ => return None,
        };
        Some(SearchResult {
            id: self.id,
            media_type,
            title: self.title.or(self.name).unwrap_or_default(),
            overview: self.overview,
            release_date: self.release_date.or(self.first_air_date),
            poster_path: self.poster_path,
            backdrop_path: self.backdrop_path,
        })
    }
}

#[derive(Deserialize)]
struct TmdbMovie {
    id: i64,
    title: String,
    overview: Option<String>,
    tagline: Option<String>,
    release_date: Option<String>,
    runtime: Option<i64>,
    vote_average: Option<f64>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    genres: Vec<Genre>,
    videos: Option<TmdbVideos>,
    images: Option<TmdbImages>,
}

#[derive(Deserialize)]
struct TmdbTv {
    id: i64,
    name: String,
    overview: Option<String>,
    tagline: Option<String>,
    first_air_date: Option<String>,
    episode_run_time: Option<Vec<i64>>,
    vote_average: Option<f64>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    genres: Vec<Genre>,
    videos: Option<TmdbVideos>,
    images: Option<TmdbImages>,
    seasons: Option<Vec<TmdbSeason>>,
}

#[derive(Deserialize)]
struct TmdbSeason {
    id: i64,
    season_number: i64,
    name: String,
    episode_count: i64,
    poster_path: Option<String>,
    air_date: Option<String>,
}

#[derive(Deserialize)]
struct TmdbVideos {
    results: Vec<TmdbVideo>,
}

#[derive(Deserialize)]
struct TmdbVideo {
    key: String,
    site: String,
    name: String,
    #[serde(rename = "type")]
    video_type: String,
}

#[derive(Deserialize)]
struct TmdbImages {
    posters: Vec<TmdbImage>,
    backdrops: Vec<TmdbImage>,
    logos: Vec<TmdbImage>,
}

#[derive(Deserialize)]
struct TmdbImage {
    file_path: String,
    width: i64,
    height: i64,
    iso_639_1: Option<String>,
    vote_average: f64,
}

// --- Conversions ---

fn convert_videos(videos: Option<TmdbVideos>) -> Vec<Video> {
    videos
        .map(|v| {
            v.results
                .into_iter()
                .map(|v| Video {
                    key: v.key,
                    site: v.site,
                    name: v.name,
                    video_type: v.video_type,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn convert_images(images: Option<TmdbImages>) -> Option<Images> {
    images.map(|i| Images {
        posters: i.posters.into_iter().map(convert_image).collect(),
        backdrops: i.backdrops.into_iter().map(convert_image).collect(),
        logos: i.logos.into_iter().map(convert_image).collect(),
    })
}

fn convert_image(i: TmdbImage) -> Image {
    Image {
        file_path: i.file_path,
        width: i.width,
        height: i.height,
        iso_639_1: i.iso_639_1,
        vote_average: i.vote_average,
    }
}

impl From<TmdbMovie> for MediaItem {
    fn from(m: TmdbMovie) -> Self {
        MediaItem {
            id: m.id,
            media_type: MediaType::Movie,
            title: m.title,
            overview: m.overview,
            tagline: m.tagline,
            release_date: m.release_date,
            runtime: m.runtime,
            rating: m.vote_average,
            poster_path: m.poster_path,
            backdrop_path: m.backdrop_path,
            genres: m.genres,
            videos: convert_videos(m.videos),
            images: convert_images(m.images),
            seasons: None,
        }
    }
}

impl From<TmdbTv> for MediaItem {
    fn from(t: TmdbTv) -> Self {
        MediaItem {
            id: t.id,
            media_type: MediaType::Tv,
            title: t.name,
            overview: t.overview,
            tagline: t.tagline,
            release_date: t.first_air_date,
            runtime: t.episode_run_time.and_then(|r| r.first().copied()),
            rating: t.vote_average,
            poster_path: t.poster_path,
            backdrop_path: t.backdrop_path,
            genres: t.genres,
            videos: convert_videos(t.videos),
            images: convert_images(t.images),
            seasons: t.seasons.map(|s| {
                s.into_iter()
                    .map(|s| Season {
                        id: s.id,
                        season_number: s.season_number,
                        name: s.name,
                        episode_count: s.episode_count,
                        poster_path: s.poster_path,
                        air_date: s.air_date,
                    })
                    .collect()
            }),
        }
    }
}

// --- Client ---

pub struct TmdbClient {
    api_key: String,
    client: HttpClient,
}

impl TmdbClient {
    pub fn new(config: &MoviesConfig, client: HttpClient) -> Self {
        Self {
            api_key: config.tmdb_api_key.clone(),
            client,
        }
    }

    pub async fn ping(&self) -> forge::Result<String> {
        let url = format!(
            "https://api.themoviedb.org/3/authentication?api_key={}",
            self.api_key
        );
        let res = self.client.get(&url).send().await?;
        let status = res.status();
        if status.is_success() {
            Ok("authenticated".into())
        } else {
            Err(forge::Error::Generic(status.to_string()))
        }
    }

    pub async fn search(&self, query: &str) -> forge::Result<Vec<SearchResult>> {
        let url = format!(
            "https://api.themoviedb.org/3/search/multi?api_key={}&query={}",
            self.api_key,
            urlencoding::encode(query)
        );
        let res = self.client.get(&url).send().await?.error_for_status()?;
        let body = res.text().await?;
        let data: TmdbMultiSearchResults = json::from_str(&body)?;
        Ok(data
            .results
            .into_iter()
            .filter_map(|r| r.into_search_result())
            .collect())
    }

    pub async fn details(&self, media_type: MediaType, id: i64) -> forge::Result<MediaItem> {
        let type_str = match media_type {
            MediaType::Movie => "movie",
            MediaType::Tv => "tv",
        };
        let url = format!(
            "https://api.themoviedb.org/3/{}/{}?api_key={}&append_to_response=videos,images",
            type_str, id, self.api_key
        );
        let res = self.client.get(&url).send().await?.error_for_status()?;
        let body = res.text().await?;

        let item = match media_type {
            MediaType::Movie => json::from_str::<TmdbMovie>(&body)?.into(),
            MediaType::Tv => json::from_str::<TmdbTv>(&body)?.into(),
        };
        Ok(item)
    }
}
