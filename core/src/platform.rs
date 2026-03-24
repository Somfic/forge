use std::{net::SocketAddr, sync::Arc};

use axum::{Json, Router, routing::get};
use tokio::signal;
use tracing::{Instrument, error, info, info_span, warn};

use crate::{AppContext, Config, HealthStatus, Module, Result};

pub struct Platform {
    config: Arc<Config>,
    modules: Vec<Box<dyn Module>>,
}

impl Platform {
    pub fn new(config: Config, modules: Vec<Box<dyn Module>>) -> Platform {
        Platform {
            config: Arc::new(config),
            modules,
        }
    }
}

impl Platform {
    pub async fn run(self) -> crate::Result<()> {
        info!("initializing event bus");
        let event_bus = crate::events::create_event_bus(&self.config);
        info!("initializing http client");
        let http = crate::http::create_client(&self.config)?;

        let mut nav_entries = Vec::new();
        let mut health_checks = Vec::new();
        let mut router = Router::new();

        for module in &self.modules {
            let span = info_span!("module", module = module.name());

            let (module_router, entry, check, report) = async {
                info!("booting");

                info!("connecting database");
                let pool = crate::db::create_module_pool(&self.config, module.as_ref()).await?;
                info!("database ready");

                let storage = crate::fs::create_storage(&self.config, module.as_ref()).await?;

                let ctx = AppContext {
                    db: pool,
                    storage,
                    config: self.config.clone(),
                    events: event_bus.clone(),
                    http: http.clone(),
                };

                let entry = module.nav_entry();
                if let Some(ref e) = entry {
                    info!("registering nav entry {}", e.label);
                }

                let check = module.live_status();
                let report = module.health_check(ctx.clone()).await?;

                if !report.is_empty() {
                    info!("health checks:");
                }

                for c in &report {
                    match c.status {
                        HealthStatus::Ok => {
                            info!("  ✓ {}: {}", c.name, c.message.as_deref().unwrap_or("ok"))
                        }
                        HealthStatus::Missing => warn!(
                            "  ✗ {} missing: {}",
                            c.name,
                            c.message.as_deref().unwrap_or("")
                        ),
                        HealthStatus::Error => error!(
                            "  ✗ {} error: {}",
                            c.name,
                            c.message.as_deref().unwrap_or("")
                        ),
                    }
                }

                let module_routes = module.routes().with_state(ctx.clone());

                info!("starting");
                module.on_start(ctx).await?;
                info!("ready");

                crate::Result::Ok((module_routes, entry, check, report))
            }
            .instrument(span)
            .await?;

            router = router.nest(&format!("/{}", module.name()).to_lowercase(), module_router);

            if let Some(entry) = entry {
                nav_entries.push(entry);
            }

            if let Some(check) = check {
                health_checks.push(check);
            }
        }

        nav_entries.sort_by_key(|e| e.order);
        let nav_entries = Arc::new(nav_entries);
        let health_checks = Arc::new(health_checks);

        let core_router = Router::new()
            .route(
                "/api/nav",
                get(move || async move { Json((*nav_entries).clone()) }),
            )
            .route(
                "/api/health",
                get({
                    move || async move {
                        let mut results = vec![];
                        for check in health_checks.iter() {
                            results.push(check.check().await);
                        }
                        Json(results)
                    }
                }),
            );

        let app = router.merge(core_router);

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("listening on http://{addr}");
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received, draining connections...");
}
