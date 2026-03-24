use crate::{Config, Error, Result};
use std::time::Duration;

pub type HttpClient = reqwest::Client;

pub(crate) fn create_client(config: &Config) -> Result<HttpClient> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(Error::HttpClientError)
}
