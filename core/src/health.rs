use async_trait::async_trait;
use serde::Serialize;
use utoipa::ToSchema;

#[async_trait]
pub trait LiveCheck: Send + Sync {
    async fn check(&self) -> LiveReport;
}

#[derive(Serialize, ToSchema, Clone)]
pub struct HealthReport {
    pub module: &'static str,
    pub checks: Vec<HealthCheck>,
}

#[derive(Serialize, ToSchema, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub requirement: Option<String>,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

#[derive(Serialize, ToSchema, Clone)]
pub struct LiveReport {
    pub module: &'static str,
    pub status: LiveStatus,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
}

#[derive(Serialize, ToSchema, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LiveStatus {
    Up,
    Degraded,
    Down,
}

#[derive(Serialize, ToSchema, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Ok,
    Missing,
    Error,
}
