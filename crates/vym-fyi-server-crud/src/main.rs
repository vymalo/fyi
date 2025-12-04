#[macro_use]
extern crate rocket;

use crate::app::{CrudApp, CrudAppBuilder};
use crate::handlers::health::health;
use crate::handlers::links::{create_link, list_links};
use mimalloc::MiMalloc;
use rocket_prometheus::PrometheusMetrics;
use tracing::info;
use vym_fyi_model::models::errors::{AppError, AppResult};
use vym_fyi_model::services::logging::setup_logging;

mod app;
mod auth;
mod handlers;
mod models;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[rocket::main]
async fn main() -> AppResult<()> {
    setup_logging("vym-fyi-server-crud")?;

    info!("starting CRUD server");

    // Build the application facade using the builder + factory patterns.
    let app: CrudApp = CrudAppBuilder::from_env()?
        .max_connections(5)
        .build()
        .await?;

    let prometheus = PrometheusMetrics::new();

    rocket::build()
        .attach(prometheus.clone())
        .manage(app)
        .mount("/", routes![health])
        .mount("/api", routes![create_link, list_links])
        .mount("/metrics", prometheus)
        .launch()
        .await
        .map_err(|e| AppError::RocketError(Box::new(e)))?;

    Ok(())
}
