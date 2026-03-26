use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{Json, Router, response::IntoResponse, routing::get};
use tokio::process::Child;
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::{Instrument, error, info, info_span, warn};

use crate::{AppContext, Config, HealthCheck, HealthStatus, LiveCheck, Module, NavEntry, Result};

pub struct Platform {
    config: Arc<Config>,
    modules: Vec<Box<dyn Module>>,
    dev: bool,
}

impl Platform {
    pub fn new(config: Config, modules: Vec<Box<dyn Module>>) -> Platform {
        Platform {
            config: Arc::new(config),
            modules,
            dev: false,
        }
    }

    pub fn dev(mut self, dev: bool) -> Self {
        self.dev = dev;
        self
    }

    pub async fn run(self) -> Result<()> {
        let event_bus = crate::events::create_event_bus(&self.config);
        let http = crate::http::create_client(&self.config)?;

        let mut router = Router::new();
        let mut nav_entries = Vec::new();
        let mut health_checks: Vec<Box<dyn LiveCheck>> = Vec::new();
        let mut dev_children: Vec<Child> = Vec::new();

        for module in &self.modules {
            let span = info_span!("module", module = module.name());
            let result = init_module(
                module.as_ref(),
                &self.config,
                &event_bus,
                &http,
                self.dev,
                router,
            )
            .instrument(span)
            .await?;

            router = result.router;

            if let Some(entry) = result.nav_entry {
                nav_entries.push(entry);
            }
            if let Some(check) = result.live_check {
                health_checks.push(check);
            }
            if let Some(child) = result.dev_child {
                dev_children.push(child);
            }
        }

        {
            let span = info_span!("module", module = "Dashboard");
            let _enter = span.enter();
            router = router.merge(core_routes(nav_entries, health_checks));
        }

        // Set fallback LAST — dev proxy or dashboard static files
        if self.dev {
            let dev_proxies: Vec<(String, crate::proxy::DevProxy)> = self
                .modules
                .iter()
                .filter_map(|m| {
                    let port = m.dev_port()?;
                    let prefix = format!("/{}", m.name()).to_lowercase();
                    Some((prefix.clone(), crate::proxy::DevProxy::new(port, "")))
                })
                .collect();

            if !dev_proxies.is_empty() {
                let proxies = Arc::new(dev_proxies);
                router = router.fallback(move |req: axum::extract::Request| {
                    let proxies = proxies.clone();
                    async move {
                        let path = req.uri().path().to_string();
                        for (prefix, proxy) in proxies.iter() {
                            if path.starts_with(prefix) || path.starts_with("/@") {
                                return crate::proxy::dev_proxy_handler(
                                    axum::extract::State(proxy.clone()),
                                    req,
                                )
                                .await
                                .into_response();
                            }
                        }
                        // Fallback: proxy to first dev server (for /@vite, /node_modules, etc.)
                        if let Some((_, proxy)) = proxies.first() {
                            return crate::proxy::dev_proxy_handler(
                                axum::extract::State(proxy.clone()),
                                req,
                            )
                            .await
                            .into_response();
                        }
                        axum::response::Response::builder()
                            .status(404)
                            .body(axum::body::Body::from("Not found"))
                            .unwrap()
                    }
                });
            }
        } else {
            router = mount_dashboard(router);
        }

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("listening on http://{addr}");
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        // Kill dev servers on shutdown
        for mut child in dev_children {
            let _ = child.kill().await;
        }

        Ok(())
    }
}

struct InitResult {
    router: Router,
    nav_entry: Option<NavEntry>,
    live_check: Option<Box<dyn LiveCheck>>,
    dev_child: Option<Child>,
}

async fn init_module(
    module: &dyn Module,
    config: &Arc<Config>,
    event_bus: &crate::EventBus,
    http: &crate::HttpClient,
    dev: bool,
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

    // Frontend: dev proxy or static files
    let prefix = format!("/{}", module.name()).to_lowercase();
    let mut dev_child = None;
    if dev {
        if let Some(port) = module.dev_port() {
            let frontend_dir =
                PathBuf::from(format!("modules/{}/frontend", module.name().to_lowercase()));
            if frontend_dir.join("package.json").exists() {
                // Kill any existing process on the dev port
                if let Ok(out) = tokio::process::Command::new("lsof")
                    .args(["-ti", &format!(":{port}")])
                    .output()
                    .await
                {
                    let pids = String::from_utf8_lossy(&out.stdout);
                    for pid in pids.split_whitespace() {
                        let _ = tokio::process::Command::new("kill").arg(pid).output().await;
                    }
                    if !pids.is_empty() {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                }

                info!("starting vite dev server on port {port}");
                match tokio::process::Command::new("bun")
                    .args([
                        "run",
                        "dev",
                        "--",
                        "--port",
                        &port.to_string(),
                        "--strictPort",
                    ])
                    .current_dir(&frontend_dir)
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .kill_on_drop(true)
                    .spawn()
                {
                    Ok(child) => dev_child = Some(child),
                    Err(e) => warn!("failed to start vite dev server: {e}"),
                }
            }

            info!("proxying ui at {prefix} → http://localhost:{port}");
        }
    }

    // Static frontend (non-dev only)
    if !dev {
        let build_dir = PathBuf::from(format!(
            "modules/{}/frontend/build",
            module.name().to_lowercase()
        ));
        if build_dir.exists() {
            info!("mounting ui at {prefix}");
            let fallback = tower_http::services::ServeFile::new(build_dir.join("index.html"));
            let service = ServeDir::new(&build_dir)
                .append_index_html_on_directories(true)
                .fallback(fallback);
            router = router.nest_service(&prefix, service);
        }
    }

    // API routes + OpenAPI spec (always)
    let api_prefix = format!("/{}/api", module.name()).to_lowercase();
    info!("mounting api at {api_prefix}");
    let (api_router, api_spec) = module.routes().split_for_parts();
    router = router.nest(&api_prefix, api_router.with_state(ctx.clone()));

    let spec_path = format!("{}/openapi.json", api_prefix);
    info!("serving openapi spec at {spec_path}");
    let spec_json = serde_json::to_value(&api_spec).unwrap();
    router = router.route(
        &spec_path,
        get(move || async move { Json(spec_json.clone()) }),
    );

    module.on_start(ctx).await?;

    Ok(InitResult {
        router,
        nav_entry,
        live_check,
        dev_child,
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
