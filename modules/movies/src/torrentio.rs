use forge::HttpClient;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize)]
struct TorrentioResponse {
    streams: Vec<TorrentioStream>,
}

#[derive(Deserialize)]
struct TorrentioStream {
    name: Option<String>,
    title: Option<String>,
    #[serde(rename = "infoHash")]
    info_hash: Option<String>,
    #[serde(rename = "fileIdx")]
    file_idx: Option<i64>,
    url: Option<String>,
}

#[derive(Serialize, Clone, ToSchema)]
pub struct Stream {
    pub name: String,
    pub title: String,
    pub info_hash: String,
    pub file_idx: i64,
}

pub struct TorrentioClient {
    client: HttpClient,
    stremio_base: String,
}

impl TorrentioClient {
    pub fn new(client: HttpClient, stremio_base: String) -> Self {
        Self {
            client,
            stremio_base,
        }
    }

    pub async fn streams_for_movie(&self, imdb_id: &str) -> forge::Result<Vec<Stream>> {
        let url = format!(
            "https://torrentio.strem.fun/stream/movie/{}.json",
            imdb_id
        );
        let res = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| forge::Error::Generic(e.to_string()))?;
        let body: TorrentioResponse = res.json().await?;

        let streams: Vec<Stream> = body
            .streams
            .into_iter()
            .filter_map(|s| {
                let info_hash = s.info_hash?;
                Some(Stream {
                    name: s.name.unwrap_or_default(),
                    title: s.title.unwrap_or_default(),
                    info_hash,
                    file_idx: s.file_idx.unwrap_or(0),
                })
            })
            .collect();

        Ok(streams)
    }

    pub async fn streams_for_episode(
        &self,
        imdb_id: &str,
        season: i64,
        episode: i64,
    ) -> forge::Result<Vec<Stream>> {
        let url = format!(
            "https://torrentio.strem.fun/stream/series/{}:{}:{}.json",
            imdb_id, season, episode
        );
        let res = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| forge::Error::Generic(e.to_string()))?;
        let body: TorrentioResponse = res.json().await?;

        let streams: Vec<Stream> = body
            .streams
            .into_iter()
            .filter_map(|s| {
                let info_hash = s.info_hash?;
                Some(Stream {
                    name: s.name.unwrap_or_default(),
                    title: s.title.unwrap_or_default(),
                    info_hash,
                    file_idx: s.file_idx.unwrap_or(0),
                })
            })
            .collect();

        Ok(streams)
    }

    /// Start a torrent on the Stremio streaming server and return the playable URL
    pub async fn start(&self, info_hash: &str, file_idx: i64) -> forge::Result<String> {
        let create_url = format!("{}/{}/create", self.stremio_base, info_hash);
        let _ = self.client.get(&create_url).send().await;
        // Serve the raw file over HTTP — browsers can play MP4/WebM directly
        Ok(format!(
            "{}/{}/{}",
            self.stremio_base, info_hash, file_idx
        ))
    }
}
