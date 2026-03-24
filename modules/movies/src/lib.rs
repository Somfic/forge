use axum::Router;
use forge::{AppContext, Module, module};
use sqlx::migrate::Migrator;

mod config;
mod tmdb;

pub struct MoviesModule;

#[module]
impl Module for MoviesModule {
    fn name(&self) -> &'static str {
        "Movies"
    }

    fn routes(&self) -> Router<AppContext> {
        Router::new()
    }

    fn migrations(&self) -> Migrator {
        sqlx::migrate!("./migrations")
    }

    async fn on_start(&self, ctx: AppContext) -> forge::Result<()> {
        let config = ctx.config.module_config::<config::MoviesConfig>("movies")?;

        Ok(())
    }
}
