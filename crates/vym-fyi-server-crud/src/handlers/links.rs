use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::app::CrudApp;

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
pub async fn create_link(
    payload: Json<CreateLinkRequest>,
    app: &State<CrudApp>,
) -> Result<(Status, Json<LinkResponse>), Status> {
    let req = payload.into_inner();
    info!(
        "Create link requested: slug={} target_url={}",
        req.slug, req.target_url
    );

    let repo = app.short_link_repository();
    let result = repo.upsert(&req.slug, &req.target_url).await.map_err(|e| {
        error!("Database error inserting/updating short link: {}", e);
        Status::InternalServerError
    })?;

    let response = LinkResponse {
        slug: result.0,
        target_url: result.1,
        active: result.2,
    };

    Ok((Status::Created, Json(response)))
}

/// List short links.
#[get("/links")]
pub async fn list_links(app: &State<CrudApp>) -> Result<Json<Vec<LinkResponse>>, Status> {
    info!("List links requested");
    let repo = app.short_link_repository();
    let rows = repo.list_all().await.map_err(|e| {
        error!("Database error listing short links: {}", e);
        Status::InternalServerError
    })?;

    let links = rows
        .into_iter()
        .map(|(slug, target_url, active)| LinkResponse {
            slug,
            target_url,
            active,
        })
        .collect();

    Ok(Json(links))
}
