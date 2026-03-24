use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, de::DeserializeOwned};

use crate::Result;

#[derive(Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub data_dir: PathBuf,
    // pub auth: AuthConfig,
    #[serde(flatten)]
    pub modules: HashMap<String, toml::Value>,
}

impl Config {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| crate::Error::ConfigReadError {
            path: path.display().to_string(),
            source: e,
        })?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn module_config<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        self.modules
            .get(key)
            .ok_or_else(|| panic!("missing config section [{key}]"))
            .and_then(|v| v.clone().try_into().map_err(Into::into))
    }
}
