use sqlx::sqlite::SqlitePoolOptions;

use crate::{Config, Error, Module, Result};

pub type Pool = sqlx::Pool<sqlx::Sqlite>;

pub(crate) async fn create_module_pool(config: &Config, module: &dyn Module) -> Result<Pool> {
    let dir = config.data_dir.join(format!("{:?}", module.name()));
    tokio::fs::create_dir_all(&dir).await?;

    let db_path = dir.join("data.db");
    let url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    // run module migrations in order
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name     TEXT PRIMARY KEY,
            checksum TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&pool)
    .await?;

    for sql in module.migrations() {
        let checksum = format!("{:x}", md5::compute(sql));
        // derive name from first line comment or use checksum
        let name = sql
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("-- "))
            .unwrap_or(&checksum)
            .to_string();

        let existing: Option<String> =
            sqlx::query_scalar("SELECT checksum FROM _migrations WHERE name = ?")
                .bind(&name)
                .fetch_optional(&pool)
                .await?;

        match existing {
            Some(ref existing_checksum) if *existing_checksum == checksum => continue,
            Some(_) => {
                return Err(Error::MigrationAlreadyApplied(
                    name,
                    format!("{:?}", module.name()),
                ));
            }
            None => {
                sqlx::query(sql).execute(&pool).await?;

                sqlx::query("INSERT INTO _migrations (name, checksum) VALUES (?, ?)")
                    .bind(&name)
                    .bind(&checksum)
                    .execute(&pool)
                    .await?;

                tracing::info!("applied migration '{}' for {:?}", name, module.name());
            }
        }
    }

    Ok(pool)
}
