use serde::Deserialize;

#[derive(Deserialize)]
pub struct MoviesConfig {
    pub tmdb_api_key: String,
    #[serde(default = "default_stremio_url")]
    pub stremio_url: String,
}

fn default_stremio_url() -> String {
    "http://127.0.0.1:11470".to_string()
}
