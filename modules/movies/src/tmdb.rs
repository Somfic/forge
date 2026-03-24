use crate::config::MoviesConfig;
use forge::{HttpClient, json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TmdbMovie {
    pub id: i64,
    pub title: String,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub release_date: Option<String>,
    pub runtime: Option<i64>,
    pub vote_average: Option<f64>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub genres: Vec<TmdbGenre>,
    pub videos: Option<TmdbVideos>,
}

#[derive(Deserialize)]
pub struct TmdbGenre {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize)]
pub struct TmdbVideos {
    pub results: Vec<TmdbVideo>,
}

#[derive(Deserialize)]
pub struct TmdbVideo {
    pub key: String,
    pub site: String,
    pub name: String,
    #[serde(rename = "type")]
    pub video_type: String,
}

#[derive(Deserialize)]
pub struct TmdbSearchResults {
    pub results: Vec<TmdbSearchResult>,
}

#[derive(Deserialize, Serialize)]
pub struct TmdbSearchResult {
    pub id: i64,
    pub title: String,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub poster_path: Option<String>,
}

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

    pub async fn search_movies(&self, query: &str) -> forge::Result<TmdbSearchResults> {
        let url = format!(
            "https://api.themoviedb.org/3/search/movie?api_key={}&query={}",
            self.api_key,
            urlencoding::encode(query)
        );
        let res = self.client.get(&url).send().await?.error_for_status()?;
        let body = res.text().await?;
        let data: TmdbSearchResults = json::from_str::<TmdbSearchResults>(&body)?;
        Ok(data)
    }

    pub async fn get_movie_details(&self, movie_id: i64) -> forge::Result<TmdbMovie> {
        let url = format!(
            "https://api.themoviedb.org/3/movie/{}?api_key={}&append_to_response=videos",
            movie_id, self.api_key
        );
        let res = self.client.get(&url).send().await?.error_for_status()?;
        let body = res.text().await?;
        let data = json::from_str::<TmdbMovie>(&body)?;
        Ok(data)
    }
}
