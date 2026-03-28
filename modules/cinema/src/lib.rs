use forge::{AppContext, Module, module};
use sqlx::migrate::Migrator;
use tracing::Instrument;
use utoipa_axum::router::OpenApiRouter;

mod config;
pub(crate) mod downloads;
mod routes;
mod streams;
mod subtitles;
mod tmdb;
pub(crate) mod torrent;

pub struct CinemaModule;

/// Generate the OpenAPI spec as JSON (used by build scripts and tests)
pub fn openapi_spec() -> utoipa::openapi::OpenApi {
    let (_, spec) = routes::router().split_for_parts();
    spec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_openapi_spec() {
        let spec = openapi_spec();
        let json = forge::json::to_string_pretty(&spec).unwrap();
        std::fs::write("frontend/openapi.json", &json).unwrap();
        println!("Wrote OpenAPI spec to frontend/openapi.json");
    }
}

#[module]
impl Module for CinemaModule {
    fn name(&self) -> &'static str {
        "Cinema"
    }

    fn dev_port(&self) -> Option<u16> {
        Some(5174)
    }

    fn routes(&self) -> OpenApiRouter<AppContext> {
        routes::router()
    }

    fn migrations(&self) -> Migrator {
        sqlx::migrate!("./migrations")
    }

    async fn on_start(&self, ctx: AppContext) -> forge::Result<()> {
        let config = ctx.config.module_config_env::<config::CinemaConfig>("cinema")?;

        torrent::TorrentEngine::init(&config, &ctx.storage, ctx.http.clone()).await?;

        let span = tracing::Span::current();
        let manager = downloads::DownloadManager::new(ctx.clone(), config);
        tokio::spawn(manager.run().instrument(span));
        Ok(())
    }

    async fn on_stop(&self) {
        torrent::TorrentEngine::get().shutdown().await;
    }

    async fn health_check(&self, ctx: AppContext) -> forge::Result<Vec<forge::HealthCheck>> {
        let config = ctx.config.module_config_env::<config::CinemaConfig>("cinema")?;
        let client = tmdb::TmdbClient::new(&config, ctx.http.clone());

        let start = std::time::Instant::now();
        let (status, message) = match client.ping().await {
            Ok(msg) => (forge::HealthStatus::Ok, msg),
            Err(e) => (forge::HealthStatus::Error, e.to_string()),
        };
        let latency_ms = Some(start.elapsed().as_millis() as u64);

        let ffmpeg_check = match tokio::process::Command::new("ffprobe").arg("-version").output().await {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                let version = version.split_whitespace().nth(2).unwrap_or("unknown");
                forge::HealthCheck {
                    name: "ffmpeg".into(),
                    status: forge::HealthStatus::Ok,
                    latency_ms: None,
                    message: Some(format!("ffprobe {version}")),
                    requirement: Some("ffmpeg + ffprobe on PATH".into()),
                }
            }
            _ => forge::HealthCheck {
                name: "ffmpeg".into(),
                status: forge::HealthStatus::Error,
                latency_ms: None,
                message: Some("ffprobe not found".into()),
                requirement: Some("ffmpeg + ffprobe on PATH".into()),
            },
        };

        Ok(vec![
            forge::HealthCheck {
                name: "TMDB API".into(),
                status,
                latency_ms,
                message: Some(message),
                requirement: Some("Valid TMDB API key".into()),
            },
            ffmpeg_check,
        ])
    }
}
