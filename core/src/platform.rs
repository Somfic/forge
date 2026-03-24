use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{Json, Router, routing::get};
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::{Instrument, error, info, info_span, warn};

use crate::{AppContext, Config, HealthCheck, HealthStatus, LiveCheck, Module, NavEntry, Result};

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

    pub async fn run(self) -> Result<()> {
        let event_bus = crate::events::create_event_bus(&self.config);
        let http = crate::http::create_client(&self.config)?;

        let mut router = Router::new();
        let mut nav_entries = Vec::new();
        let mut health_checks: Vec<Box<dyn LiveCheck>> = Vec::new();

        for module in &self.modules {
            let span = info_span!("module", module = module.name());
            let result = init_module(module.as_ref(), &self.config, &event_bus, &http, router)
                .instrument(span)
                .await?;

            router = result.router;

            if let Some(entry) = result.nav_entry {
                nav_entries.push(entry);
            }
            if let Some(check) = result.live_check {
                health_checks.push(check);
            }
        }

        {
            let span = info_span!("module", module = "Dashboard");
            let _enter = span.enter();
            router = router.merge(core_routes(nav_entries, health_checks));
            router = mount_dashboard(router);
        }

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("listening on http://{addr}");
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }
}

struct InitResult {
    router: Router,
    nav_entry: Option<NavEntry>,
    live_check: Option<Box<dyn LiveCheck>>,
}

async fn init_module(
    module: &dyn Module,
    config: &Arc<Config>,
    event_bus: &crate::EventBus,
    http: &crate::HttpClient,
    mut router: Router,
) -> Result<InitResult> {
    let pool = crate::db::create_module_pool(config, module).await?;

    let storage = crate::fs::create_storage(config, module).await?;

    let ctx = AppContext {
        db: pool,
        storage,
        config: config.clone(),
        events: event_bus.clone(),
        http: http.clone(),
    };

    let nav_entry = module.nav_entry();
    if let Some(ref e) = nav_entry {
        info!("registering nav entry {}", e.label);
    }

    let live_check = module.live_status();

    log_health_checks(&module.health_check(ctx.clone()).await?);

    // Static frontend
    let build_dir = PathBuf::from(format!(
        "modules/{}/frontend/build",
        module.name().to_lowercase()
    ));
    if build_dir.exists() {
        let prefix = format!("/{}", module.name()).to_lowercase();
        info!("mounting ui at {prefix}");
        let fallback = tower_http::services::ServeFile::new(build_dir.join("index.html"));
        let service = ServeDir::new(&build_dir)
            .append_index_html_on_directories(true)
            .fallback(fallback);
        router = router.nest_service(&prefix, service);
    }

    // API routes
    let api_prefix = format!("/{}/api", module.name()).to_lowercase();
    info!("mounting api at {api_prefix}");
    router = router.nest(&api_prefix, module.routes().with_state(ctx.clone()));

    module.on_start(ctx).await?;

    Ok(InitResult {
        router,
        nav_entry,
        live_check,
    })
}

fn log_health_checks(checks: &[HealthCheck]) {
    if checks.is_empty() {
        return;
    }

    info!("health checks:");
    for c in checks {
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
}

fn core_routes(nav_entries: Vec<NavEntry>, health_checks: Vec<Box<dyn LiveCheck>>) -> Router {
    let nav_entries = Arc::new(nav_entries);
    let health_checks = Arc::new(health_checks);

    let prefix = "/api";
    info!("mounting api at {prefix}");

    Router::new()
        .route(
            format!("{}/nav", prefix).as_str(),
            get(move || async move { Json((*nav_entries).clone()) }),
        )
        .route(
            format!("{}/health", prefix).as_str(),
            get(move || async move {
                let mut results = vec![];
                for check in health_checks.iter() {
                    results.push(check.check().await);
                }
                Json(results)
            }),
        )
}

fn mount_dashboard(mut router: Router) -> Router {
    let dashboard_dir = PathBuf::from("frontend/apps/dashboard/build");
    if dashboard_dir.exists() {
        info!("mounting ui at /");
        router = router
            .fallback_service(ServeDir::new(&dashboard_dir).append_index_html_on_directories(true));
    }
    router
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
