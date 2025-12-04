use crate::app::{CrudApp, CrudAppBuilder};
use crate::handlers::health::health;
use crate::handlers::links::{create_link, list_links};
use axum::{
    Router,
    routing::{get, post},
};
use axum_prometheus::PrometheusMetricLayer;
use mimalloc::MiMalloc;
use tokio::net::TcpListener;
use tracing::info;
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::services::config::bind_addr_from_env;
use vym_fyi_model::services::logging::setup_logging;

mod app;
mod auth;
mod handlers;
mod models;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> AppResult<()> {
    setup_logging("vym-fyi-server-crud")?;

    info!("starting CRUD server");

    // Build the application facade using the builder + factory patterns.
    let app: CrudApp = CrudAppBuilder::from_env()?
        .max_connections(5)
        .build()
        .await?;

    let (prometheus_layer, prometheus_handle) = PrometheusMetricLayer::pair();
    let metrics_handle = prometheus_handle.clone();

    let router = Router::new()
        .route("/health", get(health))
        .route("/api/links", post(create_link).get(list_links))
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
