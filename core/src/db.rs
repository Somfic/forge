use sqlx::sqlite::SqlitePoolOptions;
use tracing::info;

use crate::{Config, Error, Module, Result};

pub type Pool = sqlx::Pool<sqlx::Sqlite>;

pub(crate) async fn create_module_pool(config: &Config, module: &dyn Module) -> Result<Pool> {
    let dir = config
        .data_dir
        .join(module.name().to_lowercase())
        .join("db");
    tokio::fs::create_dir_all(&dir).await?;

    let db_path = dir.join("data.db");
    let url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    let migrator = module.migrations();

    migrator.run(&pool).await?;

    Ok(pool)
}
