use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{Config, Module, Result};

#[derive(Clone)]
pub struct Storage(Arc<PathBuf>);

impl Storage {
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn join(&self, p: impl AsRef<Path>) -> PathBuf {
        self.0.join(p)
    }
}

pub(crate) async fn create_storage(config: &Config, module: &dyn Module) -> Result<Storage> {
    let path = config
        .data_dir
        .join(module.name().to_lowercase())
        .join("fs");
    tokio::fs::create_dir_all(&path).await?;
    Ok(Storage(Arc::new(path)))
}
