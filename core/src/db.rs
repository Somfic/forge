use sqlx::any::AnyPoolOptions;
use tracing::info;

use crate::{Config, Module, Result};

pub type Pool = sqlx::AnyPool;

pub(crate) async fn create_module_pool(config: &Config, module: &dyn Module) -> Result<Pool> {
    sqlx::any::install_default_drivers();

    let url = if let Some(ref database_url) = config.database_url {
        database_url.clone()
    } else if let Ok(database_url) = std::env::var("FORGE_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
    {
        database_url
    } else {
        let dir = config
            .data_dir
            .join(module.name().to_lowercase())
            .join("db");
        tokio::fs::create_dir_all(&dir).await?;

        let db_path = dir.join("data.db");
        format!("sqlite:{}?mode=rwc", db_path.display())
    };

    info!("connecting to database at {url}");

    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    let migrator = module.migrations();
    migrator.run(&pool).await?;

    Ok(pool)
}
