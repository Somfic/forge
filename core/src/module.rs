use async_trait::async_trait;
use axum::Router;
use serde::Serialize;
use specta::Type;
use sqlx::migrate::Migrator;

use crate::{AppContext, HealthCheck, LiveCheck, Result};

#[async_trait]
pub trait Module: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn nav_entry(&self) -> Option<NavEntry> {
        None
    }

    fn routes(&self) -> Router<AppContext>;

    fn migrations(&self) -> Migrator;

    async fn on_start(&self, ctx: AppContext) -> Result<()>;

    async fn health_check(&self, ctx: AppContext) -> Result<Vec<HealthCheck>> {
        Ok(vec![])
    }

    fn live_status(&self) -> Option<Box<dyn LiveCheck>> {
        None
    }
}

#[derive(Serialize, Type, Clone)]
pub struct NavEntry {
    pub label: String,
    pub path: String,
    pub icon: String,
    pub order: i32,
}
