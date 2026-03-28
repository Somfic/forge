use std::env;

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
    pub torrent_port: u16,
    #[serde(default = "default_dht_enabled")]
    pub use_dht: bool,
}

impl forge::EnvOverride for CinemaConfig {
    fn apply_env_overrides(&mut self) {
        if let Ok(v) = env::var("FORGE_TMDB_API_KEY") {
            self.tmdb_api_key = v;
        }
        if let Ok(v) = env::var("FORGE_STREAM_SOURCES") {
            self.stream_sources = v.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(v) = env::var("FORGE_SUBTITLE_LANGUAGES") {
            self.subtitle_languages = v.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(v) = env::var("FORGE_MAX_CONCURRENT_DOWNLOADS") {
            if let Ok(n) = v.parse() {
                self.max_concurrent_downloads = n;
            }
        }
        if let Ok(v) = env::var("FORGE_TORRENT_PORT") {
            if let Ok(n) = v.parse() {
                self.torrent_port = n;
            }
        }
        if let Ok(v) = env::var("FORGE_USE_DHT") {
            if let Ok(b) = v.parse() {
                self.use_dht = b;
            }
        }
    }
}

fn default_max_concurrent_downloads() -> usize {
    2
}

fn default_subtitle_languages() -> Vec<String> {
    vec!["en".to_string()]
}

fn default_stream_sources() -> Vec<String> {
    vec!["https://torrentio.strem.fun".to_string()]
}

fn default_torrent_listen_port() -> u16 {
    6881
}

fn default_dht_enabled() -> bool {
    true
}
