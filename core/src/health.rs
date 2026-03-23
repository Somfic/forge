use async_trait::async_trait;
use serde::Serialize;
use specta::Type;

use crate::ModuleName;

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthStatus;
}

#[derive(Serialize, Type, Clone)]
pub struct HealthStatus {
    pub module: ModuleName,
    pub status: ServiceStatus,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
}

#[derive(Serialize, Type, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    Up,
    Degraded,
    Down,
}
