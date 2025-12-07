use axum::Json;

use crate::models::Health;
use tracing::debug;

pub async fn health() -> Json<Health> {
    debug!("GET /health requested");
    Json(Health::ok())
}
