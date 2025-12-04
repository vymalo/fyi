#[macro_use]
extern crate rocket;

use crate::app::{RedirectApp, RedirectAppBuilder};
use crate::handlers::health::health;
use crate::handlers::short_link::redirect_short_link;
use mimalloc::MiMalloc;
use rocket_prometheus::PrometheusMetrics;
use tracing::info;
use vym_fyi_model::models::errors::{AppError, AppResult};
use vym_fyi_model::services::logging::setup_logging;

mod app;
mod handlers;
mod models;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[rocket::main]
async fn main() -> AppResult<()> {
    setup_logging("vym-fyi-server-redirect")?;

    info!("starting redirect server");

    let app: RedirectApp = RedirectAppBuilder::from_env()?
        .max_connections(5)
        .build()
        .await?;

    let prometheus = PrometheusMetrics::new();

    rocket::build()
        .attach(prometheus.clone())
        .manage(app)
        .mount("/", routes![health, redirect_short_link])
        .mount("/metrics", prometheus)
        .launch()
        .await
        .map_err(|e| AppError::RocketError(Box::new(e)))?;

    Ok(())
}
