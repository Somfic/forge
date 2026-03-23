use async_trait::async_trait;
use axum::Router;
use serde::Serialize;
use specta::Type;

use crate::{AppContext, HealthCheck, Result};

#[derive(Serialize, Type, Clone, Debug)]
pub enum ModuleName {}

#[async_trait]
pub trait Module: Send + Sync + 'static {
    fn name(&self) -> ModuleName;

    fn nav_entry(&self) -> Option<NavEntry> {
        None
    }

    fn routes(&self) -> Router<AppContext>;

    fn migrations(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn on_start(&self, ctx: AppContext) -> Result<()> {
        Ok(())
    }

    fn health_check(&self) -> Option<Box<dyn HealthCheck>> {
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
