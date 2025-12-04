use chrono::{DateTime, Utc};
use rocket::State;
use rocket::form::FromForm;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use sqlx::Row;
use tracing::{error, info};

use crate::app::CrudApp;
use crate::auth::ApiKeyAuth;

#[derive(Deserialize)]
pub struct CreateLinkRequest {
    /// Optional slug. If omitted or empty, the server will generate a random slug.
    pub slug: Option<String>,
    pub target_url: String,
}

#[derive(Serialize)]
pub struct LinkResponse {
    pub slug: String,
    pub target_url: String,
    pub active: bool,
}

/// Query parameters for listing links.
#[derive(FromForm)]
pub struct ListLinksQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub slug: Option<String>,
    pub target_contains: Option<String>,
    pub active: Option<bool>,
    pub created_before: Option<String>,
    pub created_after: Option<String>,
    pub expires_before: Option<String>,
    pub expires_after: Option<String>,
}

/// Create a short link (skeleton, no persistence yet).
#[post("/links", data = "<payload>")]
pub async fn create_link(
    payload: Json<CreateLinkRequest>,
    auth: ApiKeyAuth,
    app: &State<CrudApp>,
) -> Result<(Status, Json<LinkResponse>), Status> {
    let tenant_id = auth.tenant_id.ok_or(Status::Forbidden)?;

    let req = payload.into_inner();
    let repo = app.short_link_repository();
    let result = match req.slug.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(slug) => {
            info!(
                "Create link requested with slug={} target_url={}",
                slug, req.target_url
            );
            repo.upsert(slug, &req.target_url, tenant_id).await
        }
        None => {
            info!(
                "Create link requested without slug; generating slug for target_url={}",
                req.target_url
            );
            repo.create_with_generated_slug(&req.target_url, 6, tenant_id)
                .await
        }
    }
    .map_err(|e| {
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
///
/// - Non-master keys: only links for the caller's tenant are returned.
/// - Master key: all links across all tenants are returned.
/// Pagination:
///   - `page` (1-based, default 1)
///   - `per_page` (items per page, default 20, max 100)
/// Additional filters:
///   - `slug` (exact match)
///   - `target_contains` (case-insensitive substring match on target URL)
///   - `active` (true/false)
///   - `created_before` / `created_after` (RFC3339 timestamps on creation time)
///   - `expires_before` / `expires_after` (RFC3339 timestamps on expiry time)
#[get("/links?<query..>")]
pub async fn list_links(
    auth: ApiKeyAuth,
    app: &State<CrudApp>,
    query: ListLinksQuery,
) -> Result<Json<Vec<LinkResponse>>, Status> {
    fn parse_rfc3339_opt(
        label: &str,
        value: &Option<String>,
    ) -> Result<Option<DateTime<Utc>>, Status> {
        match value.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            None => Ok(None),
            Some(raw) => DateTime::parse_from_rfc3339(raw)
                .map(|dt| dt.with_timezone(&Utc))
                .map(Some)
                .map_err(|e| {
                    error!("Invalid {} timestamp '{}': {}", label, raw, e);
                    Status::BadRequest
                }),
        }
    }

    let created_before = parse_rfc3339_opt("created_before", &query.created_before)?;
    let created_after = parse_rfc3339_opt("created_after", &query.created_after)?;
    let expires_before = parse_rfc3339_opt("expires_before", &query.expires_before)?;
    let expires_after = parse_rfc3339_opt("expires_after", &query.expires_after)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let limit = per_page as i64;
    let offset = ((page - 1) as i64) * (per_page as i64);

    info!(
        "List links requested (page={}, per_page={}, is_master={})",
        page, per_page, auth.is_master
    );

    let mut qb = QueryBuilder::<sqlx::Postgres>::new(
        "SELECT slug, target_url, is_active FROM short_links WHERE ",
    );

    if auth.is_master {
        qb.push("TRUE");
    } else {
        let tenant_id = auth.tenant_id.ok_or(Status::Forbidden)?;
        qb.push("tenant_id = ").push_bind(tenant_id);
    }

    if let Some(slug) = query
        .slug
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        qb.push(" AND slug = ").push_bind(slug);
    }

    if let Some(cont) = query
        .target_contains
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        qb.push(" AND target_url ILIKE ")
            .push_bind(format!("%{}%", cont));
    }

    if let Some(active) = query.active {
        qb.push(" AND is_active = ").push_bind(active);
    }

    if let Some(dt) = created_before {
        qb.push(" AND created_at < ").push_bind(dt);
    }

    if let Some(dt) = created_after {
        qb.push(" AND created_at > ").push_bind(dt);
    }

    if let Some(dt) = expires_before {
        qb.push(" AND expires_at < ").push_bind(dt);
    }

    if let Some(dt) = expires_after {
        qb.push(" AND expires_at > ").push_bind(dt);
    }

    qb.push(" ORDER BY created_at DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let query = qb.build();

    let rows = query.fetch_all(&app._pool).await.map_err(|e| {
        error!("Database error listing short links: {}", e);
        Status::InternalServerError
    })?;

    let links = rows
        .into_iter()
        .map(|row| LinkResponse {
            slug: row.get("slug"),
            target_url: row.get("target_url"),
            active: row.get::<bool, _>("is_active"),
        })
        .collect();

    Ok(Json(links))
}
