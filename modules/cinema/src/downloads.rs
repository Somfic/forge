use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use utoipa::ToSchema;

use forge::AppContext;

use crate::config::CinemaConfig;
use crate::torrentio::StremioClient;

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct Download {
    pub id: i64,
    pub media_type: String,
    pub tmdb_id: i64,
    pub title: String,
    pub poster_path: Option<String>,
    pub season: i64,
    pub episode: i64,
    pub resolution: Option<String>,
    pub info_hash: String,
    pub file_idx: i64,
    pub file_path: String,
    pub total_bytes: Option<i64>,
    pub downloaded_bytes: i64,
    pub status: String,
    pub error: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

pub struct DownloadManager {
    ctx: AppContext,
    config: CinemaConfig,
    semaphore: Arc<Semaphore>,
}

impl DownloadManager {
    pub fn new(ctx: AppContext, config: CinemaConfig) -> Self {
        let permits = config.max_concurrent_downloads;
        Self {
            ctx,
            config,
            semaphore: Arc::new(Semaphore::new(permits)),
        }
    }

    pub async fn run(self) {
        // Reset interrupted downloads on startup
        let reset = sqlx::query("UPDATE downloads SET status = 'queued' WHERE status = 'downloading'")
            .execute(&self.ctx.db)
            .await;
        if let Ok(r) = &reset {
            if r.rows_affected() > 0 {
                tracing::info!(count = r.rows_affected(), "Reset interrupted downloads to queued");
            }
        }
        tracing::info!("Download manager started");

        let mut rx = self.ctx.events.subscribe();

        loop {
            // Fetch next queued download
            let queued = sqlx::query_as::<_, Download>(
                "SELECT * FROM downloads WHERE status = 'queued' ORDER BY created_at ASC LIMIT 1",
            )
            .fetch_optional(&self.ctx.db)
            .await;

            if let Ok(Some(download)) = queued {
                let permit = self.semaphore.clone().acquire_owned().await;
                if let Ok(permit) = permit {
                    let ctx = self.ctx.clone();
                    let stremio_url = self.config.stremio_url.clone();
                    tokio::spawn(async move {
                        download_file(ctx, &stremio_url, download).await;
                        drop(permit);
                    });
                    continue; // Check for more immediately
                }
            }

            // Wait for a wake event or poll every 10 seconds
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {}
                msg = rx.recv() => {
                    if let Ok(event) = msg {
                        if event.topic == "download:enqueue" {
                            continue;
                        }
                    }
                }
            }
        }
    }
}

async fn download_file(ctx: AppContext, stremio_url: &str, download: Download) {
    let id = download.id;
    tracing::info!(
        id,
        title = download.title,
        file = download.file_path,
        "Starting download"
    );

    let _ = sqlx::query("UPDATE downloads SET status = 'downloading' WHERE id = ?")
        .bind(id)
        .execute(&ctx.db)
        .await;

    match do_download(&ctx, stremio_url, &download).await {
        Ok(()) => {
            tracing::info!(id, title = download.title, "Download completed");
            let _ = sqlx::query(
                "UPDATE downloads SET status = 'completed', completed_at = datetime('now') WHERE id = ?",
            )
            .bind(id)
            .execute(&ctx.db)
            .await;
        }
        Err(e) => {
            tracing::error!(id, title = download.title, error = %e, "Download failed");
            let _ = sqlx::query("UPDATE downloads SET status = 'failed', error = ? WHERE id = ?")
                .bind(e.to_string())
                .bind(id)
                .execute(&ctx.db)
                .await;
        }
    }
}

async fn do_download(
    ctx: &AppContext,
    stremio_url: &str,
    download: &Download,
) -> forge::Result<()> {
    let stremio = StremioClient::new(ctx.http.clone(), stremio_url.to_string());
    let url = stremio.start(&download.info_hash, download.file_idx).await?;

    // Use a client with no overall timeout (just connect timeout)
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .read_timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| forge::Error::Generic(e.to_string()))?;

    // Retry a few times — Stremio may need time to start the torrent
    let mut resp = None;
    for attempt in 0..5 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
        match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => {
                resp = Some(r);
                break;
            }
            Ok(r) => {
                tracing::warn!("Download attempt {}: status {}", attempt + 1, r.status());
            }
            Err(e) => {
                tracing::warn!("Download attempt {}: {}", attempt + 1, e);
            }
        }
    }

    let resp = resp.ok_or_else(|| {
        forge::Error::Generic("Failed to connect to stream after retries".into())
    })?;

    // Get total size from Content-Length
    let total_bytes = resp.content_length().map(|v| v as i64);
    if let Some(total) = total_bytes {
        tracing::info!(
            id = download.id,
            total_mb = total / 1_000_000,
            "Download size determined"
        );
        let _ = sqlx::query("UPDATE downloads SET total_bytes = ? WHERE id = ?")
            .bind(total)
            .bind(download.id)
            .execute(&ctx.db)
            .await;
    } else {
        tracing::warn!(id = download.id, "No Content-Length header — progress will be unknown");
    }

    // Create parent directories
    let full_path = ctx.storage.join(&download.file_path);
    if let Some(parent) = full_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Stream to file with buffered writer
    let file = tokio::fs::File::create(&full_path).await?;
    let mut writer = tokio::io::BufWriter::with_capacity(256 * 1024, file);
    let mut downloaded: i64 = 0;
    let mut last_db_update = Instant::now();

    let mut stream = resp.bytes_stream();
    use futures::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| forge::Error::Generic(e.to_string()))?;
        writer.write_all(&chunk).await?;
        downloaded += chunk.len() as i64;

        // Update DB every 3 seconds
        if last_db_update.elapsed().as_secs() >= 3 {
            let pct = total_bytes.map(|t| downloaded * 100 / t).unwrap_or(0);
            tracing::debug!(
                id = download.id,
                downloaded_mb = downloaded / 1_000_000,
                pct,
                "Download progress"
            );

            let _ = sqlx::query("UPDATE downloads SET downloaded_bytes = ? WHERE id = ?")
                .bind(downloaded)
                .bind(download.id)
                .execute(&ctx.db)
                .await;

            // Check if cancelled
            let status: Option<(String,)> =
                sqlx::query_as("SELECT status FROM downloads WHERE id = ?")
                    .bind(download.id)
                    .fetch_optional(&ctx.db)
                    .await
                    .unwrap_or(None);

            if let Some((s,)) = status {
                if s == "cancelled" {
                    let _ = tokio::fs::remove_file(&full_path).await;
                    return Err(forge::Error::Generic("Download cancelled".into()));
                }
            }

            last_db_update = Instant::now();
        }
    }

    writer.flush().await?;

    // Final progress update
    let _ = sqlx::query("UPDATE downloads SET downloaded_bytes = ? WHERE id = ?")
        .bind(downloaded)
        .bind(download.id)
        .execute(&ctx.db)
        .await;

    Ok(())
}
