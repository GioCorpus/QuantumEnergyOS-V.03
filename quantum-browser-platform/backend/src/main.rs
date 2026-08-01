use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing_subscriber::{fmt, filter::EnvFilter};

use qb_platform_backend::handlers::{health, launch_dashboard, open_browser};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // logging
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();

    // When running tests, skip launching the HTTP server to avoid compilation/runtime differences
    #[cfg(not(test))]
    {
        let app = Router::<()>::new()
            .route("/health", get(health))
            .route("/api/launch-dashboard", axum::routing::post(launch_dashboard))
            .route("/api/open-browser/:name", get(open_browser));

        let addr = SocketAddr::from(([127, 0, 0, 1], 4607));
        tracing::info!(%addr, "qb_platform_backend (scaffold) — server not started; use cargo run to start a full server");
        // In the scaffold we avoid starting the full axum/hyper server to keep compilation simple across environments.
        // Production code should bind and serve the router. Here we await a shutdown signal so the process stays alive when run.
        tokio::signal::ctrl_c().await.expect("failed to listen for shutdown");
    }

    Ok(())
}
