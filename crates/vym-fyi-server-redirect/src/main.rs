use crate::app::{RedirectApp, RedirectAppBuilder};
use crate::handlers::health::health;
use crate::handlers::short_link::redirect_short_link;
use axum::{Router, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use mimalloc::MiMalloc;
use tokio::net::TcpListener;
use tracing::info;
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::services::config::bind_addr_from_env;
use vym_fyi_model::services::logging::setup_logging;

mod app;
mod handlers;
mod models;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> AppResult<()> {
    setup_logging("vym-fyi-server-redirect")?;

    info!("starting redirect server");

    let app: RedirectApp = RedirectAppBuilder::from_env()?
        .max_connections(5)
        .build()
        .await?;

    let (prometheus_layer, prometheus_handle) = PrometheusMetricLayer::pair();
    let metrics_handle = prometheus_handle.clone();

    let router = Router::new()
        .route("/health", get(health))
        .route("/{slug}", get(redirect_short_link))
        .route(
            "/metrics",
            get(move || async move { metrics_handle.render() }),
        )
        .with_state(app.clone())
        .layer(prometheus_layer);

    let addr = bind_addr_from_env(8000)?;
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr().unwrap_or(addr);

    info!("listening on {}", local_addr);

    axum::serve(listener, router).await?;

    Ok(())
}
