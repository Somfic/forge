use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, de::DeserializeOwned};

use crate::Result;

/// Implement this on module configs to allow env var overrides.
pub trait EnvOverride {
    fn apply_env_overrides(&mut self);
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    // pub auth: AuthConfig,
    #[serde(flatten)]
    pub modules: HashMap<String, toml::Value>,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("./data/")
}

impl Config {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        match std::fs::read_to_string(path) {
            Ok(content) => Ok(toml::from_str(&content)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(toml::from_str("")?),
            Err(e) => Err(crate::Error::ConfigReadError {
                path: path.display().to_string(),
                source: e,
            }),
        }
    }

    pub fn module_config<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        match self.modules.get(key) {
            Some(v) => Ok(v.clone().try_into()?),
            None => Ok(toml::from_str("")?),
        }
    }

    pub fn module_config_env<T: DeserializeOwned + EnvOverride>(&self, key: &str) -> Result<T> {
        let mut config: T = self.module_config(key)?;
        config.apply_env_overrides();
        Ok(config)
    }
}
