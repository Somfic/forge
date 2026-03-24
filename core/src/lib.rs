use std::sync::Arc;

use reqwest::Client;

mod config;
pub use config::*;

mod db;
pub use db::*;

mod events;
pub use events::*;

mod http;
pub use http::*;

mod fs;
pub use fs::*;

mod error;
pub use error::*;

mod module;
pub use module::*;

mod health;
pub use health::*;

mod platform;
pub use platform::*;

mod logging;
pub use logging::*;

pub use async_trait::async_trait as module;
pub use serde_json as json;

#[derive(Clone)]
pub struct AppContext {
    pub db: Pool,
    pub storage: Storage,
    pub config: Arc<Config>,
    pub events: EventBus,
    pub http: Client,
}
