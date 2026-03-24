use serde::Deserialize;

#[derive(Deserialize)]
pub struct MoviesConfig {
    pub tmdb_api_key: String,
}
