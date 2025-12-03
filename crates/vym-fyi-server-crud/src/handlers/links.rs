use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
pub struct CreateLinkRequest {
    pub slug: String,
    pub target_url: String,
}

#[derive(Serialize)]
pub struct LinkResponse {
    pub slug: String,
    pub target_url: String,
    pub active: bool,
}

/// Create a short link (skeleton, no persistence yet).
#[post("/links", data = "<payload>")]
pub async fn create_link(payload: Json<CreateLinkRequest>) -> (Status, Json<LinkResponse>) {
    let req = payload.into_inner();
    info!(
        "Create link requested: slug={} target_url={}",
        req.slug, req.target_url
    );

    let response = LinkResponse {
        slug: req.slug,
        target_url: req.target_url,
        active: true,
    };

    (Status::Created, Json(response))
}

/// List short links (skeleton, returns an empty list for now).
#[get("/links")]
pub async fn list_links() -> Json<Vec<LinkResponse>> {
    info!("List links requested");
    Json(Vec::new())
}
