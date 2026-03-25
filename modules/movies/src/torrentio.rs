use forge::HttpClient;

pub struct StremioClient {
    client: HttpClient,
    stremio_base: String,
}

impl StremioClient {
    pub fn new(client: HttpClient, stremio_base: String) -> Self {
        Self {
            client,
            stremio_base,
        }
    }

    /// Start a torrent on the Stremio streaming server and return the playable URL
    pub async fn start(&self, info_hash: &str, file_idx: i64) -> forge::Result<String> {
        let create_url = format!("{}/{}/create", self.stremio_base, info_hash);
        let _ = self.client.get(&create_url).send().await;
        Ok(format!("{}/{}/{}", self.stremio_base, info_hash, file_idx))
    }
}
