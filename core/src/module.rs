use async_trait::async_trait;
use serde::Serialize;
use sqlx::migrate::Migrator;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

use crate::{AppContext, HealthCheck, LiveCheck, Result};

#[async_trait]
pub trait Module: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn nav_entry(&self) -> Option<NavEntry> {
        None
    }

    fn routes(&self) -> OpenApiRouter<AppContext>;

    fn migrations(&self) -> Migrator;

    async fn on_start(&self, ctx: AppContext) -> Result<()>;

    async fn health_check(&self, ctx: AppContext) -> Result<Vec<HealthCheck>> {
        Ok(vec![])
    }

    fn live_status(&self) -> Option<Box<dyn LiveCheck>> {
        None
    }

    /// Port for the Vite dev server (used with `--dev` for reverse proxying)
    fn dev_port(&self) -> Option<u16> {
        None
    }
}

#[derive(Serialize, ToSchema, Clone)]
pub struct NavEntry {
    pub label: String,
    pub path: String,
    pub icon: String,
    pub order: i32,
}
