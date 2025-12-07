use crate::models::health::Health;
use axum::Json;
use tracing::debug;

pub async fn health() -> Json<Health> {
    debug!("GET /health requested");
    Json(Health::new())
}
