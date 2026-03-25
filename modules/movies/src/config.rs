use serde::Deserialize;

#[derive(Deserialize)]
pub struct MoviesConfig {
    pub tmdb_api_key: String,
    #[serde(default = "default_stremio_url")]
    pub stremio_url: String,
    #[serde(default = "default_stream_sources")]
    pub stream_sources: Vec<String>,
}

fn default_stremio_url() -> String {
    "http://127.0.0.1:11470".to_string()
}

fn default_stream_sources() -> Vec<String> {
    vec![
        "https://torrentio.strem.fun".to_string(),
        "https://mediafusion.elfhosted.com".to_string(),
    ]
}
