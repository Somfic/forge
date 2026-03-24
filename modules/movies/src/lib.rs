use forge::{AppContext, Module, module};
use sqlx::migrate::Migrator;
use utoipa_axum::router::OpenApiRouter;

mod config;
mod routes;
mod tmdb;
mod torrentio;

pub struct MoviesModule;

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
impl Module for MoviesModule {
    fn name(&self) -> &'static str {
        "Movies"
    }

    fn routes(&self) -> OpenApiRouter<AppContext> {
        routes::router()
    }

    fn migrations(&self) -> Migrator {
        sqlx::migrate!("./migrations")
    }

    async fn on_start(&self, ctx: AppContext) -> forge::Result<()> {
        let config = ctx.config.module_config::<config::MoviesConfig>("movies")?;

        Ok(())
    }

    async fn health_check(&self, ctx: AppContext) -> forge::Result<Vec<forge::HealthCheck>> {
        let config = ctx.config.module_config::<config::MoviesConfig>("movies")?;
        let client = tmdb::TmdbClient::new(&config, ctx.http.clone());

        let start = std::time::Instant::now();
        let (status, message) = match client.ping().await {
            Ok(msg) => (forge::HealthStatus::Ok, msg),
            Err(e) => (forge::HealthStatus::Error, e.to_string()),
        };
        let latency_ms = Some(start.elapsed().as_millis() as u64);

        Ok(vec![forge::HealthCheck {
            name: "TMDB API".into(),
            status,
            latency_ms,
            message: Some(message),
            requirement: Some("Valid TMDB API key".into()),
        }])
    }
}
