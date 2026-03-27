use serde::Deserialize;

#[derive(Deserialize)]
pub struct CinemaConfig {
    pub tmdb_api_key: String,
    #[serde(default = "default_stream_sources")]
    pub stream_sources: Vec<String>,
    #[serde(default = "default_subtitle_languages")]
    pub subtitle_languages: Vec<String>,
    #[serde(default = "default_max_concurrent_downloads")]
    pub max_concurrent_downloads: usize,
    #[serde(default = "default_torrent_listen_port")]
    pub torrent_listen_port: u16,
    #[serde(default = "default_dht_enabled")]
    pub dht_enabled: bool,
}

fn default_max_concurrent_downloads() -> usize {
    2
}

fn default_subtitle_languages() -> Vec<String> {
    vec!["en".to_string()]
}

fn default_stream_sources() -> Vec<String> {
    vec![
        "https://torrentio.strem.fun".to_string(),
        "https://mediafusion.elfhosted.com".to_string(),
    ]
}

fn default_torrent_listen_port() -> u16 {
    6881
}

fn default_dht_enabled() -> bool {
    true
}
