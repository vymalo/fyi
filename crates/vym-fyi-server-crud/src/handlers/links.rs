use async_trait::async_trait;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use sqlx::Row;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use vym_fyi_model::models::errors::AppError;
use vym_fyi_model::services::repos::ShortLinkRepository;

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

type LinkCreationResult = vym_fyi_model::models::errors::AppResult<(String, String, bool)>;

/// Contract for link persistence used by the creation strategies.
#[async_trait]
trait LinkRepository: Send + Sync {
    async fn upsert(&self, slug: &str, target_url: &str, tenant_id: Uuid) -> LinkCreationResult;

    async fn create_with_generated_slug(
        &self,
        target_url: &str,
        min_len: usize,
        tenant_id: Uuid,
    ) -> LinkCreationResult;
}

#[async_trait]
impl LinkRepository for ShortLinkRepository {
    async fn upsert(&self, slug: &str, target_url: &str, tenant_id: Uuid) -> LinkCreationResult {
        ShortLinkRepository::upsert(self, slug, target_url, tenant_id).await
    }

    async fn create_with_generated_slug(
        &self,
        target_url: &str,
        min_len: usize,
        tenant_id: Uuid,
    ) -> LinkCreationResult {
        ShortLinkRepository::create_with_generated_slug(self, target_url, min_len, tenant_id).await
    }
}

#[async_trait]
trait LinkCreationStrategy: Send + Sync {
    fn label(&self) -> &'static str;

    async fn create(
        &self,
        repo: &(dyn LinkRepository + Send + Sync),
        target_url: &str,
        tenant_id: Uuid,
    ) -> LinkCreationResult;
}

struct ProvidedSlugStrategy {
    slug: String,
}

impl ProvidedSlugStrategy {
    fn new(slug: String) -> Self {
        Self { slug }
    }
}

#[async_trait]
impl LinkCreationStrategy for ProvidedSlugStrategy {
    fn label(&self) -> &'static str {
        "provided_slug"
    }

    async fn create(
        &self,
        repo: &(dyn LinkRepository + Send + Sync),
        target_url: &str,
        tenant_id: Uuid,
    ) -> LinkCreationResult {
        repo.upsert(&self.slug, target_url, tenant_id).await
    }
}

struct GeneratedSlugStrategy {
    min_len: usize,
}

impl GeneratedSlugStrategy {
    fn new(min_len: usize) -> Self {
        Self { min_len }
    }
}

#[async_trait]
impl LinkCreationStrategy for GeneratedSlugStrategy {
    fn label(&self) -> &'static str {
        "generated_slug"
    }

    async fn create(
        &self,
        repo: &(dyn LinkRepository + Send + Sync),
        target_url: &str,
        tenant_id: Uuid,
    ) -> LinkCreationResult {
        repo.create_with_generated_slug(target_url, self.min_len, tenant_id)
            .await
    }
}

/// Query parameters for listing links.
#[derive(Deserialize)]
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

/// List short links.
pub async fn list_links(
    State(app): State<CrudApp>,
    auth: ApiKeyAuth,
    Query(query): Query<ListLinksQuery>,
) -> Result<Json<Vec<LinkResponse>>, StatusCode> {
    fn parse_rfc3339_opt(
        label: &str,
        value: &Option<String>,
    ) -> Result<Option<DateTime<Utc>>, StatusCode> {
        match value.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            None => Ok(None),
            Some(raw) => DateTime::parse_from_rfc3339(raw)
                .map(|dt| dt.with_timezone(&Utc))
                .map(Some)
                .map_err(|e| {
                    error!("Invalid {} timestamp '{}': {}", label, raw, e);
                    StatusCode::BAD_REQUEST
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

    debug!(
        "List links requested (page={}, per_page={}, is_master={}, tenant_id={:?})",
        page, per_page, auth.is_master, auth.tenant_id
    );

    let mut qb = QueryBuilder::<sqlx::Postgres>::new(
        "SELECT slug, target_url, is_active FROM short_links WHERE ",
    );

    if auth.is_master {
        qb.push("TRUE");
    } else {
        let tenant_id = auth.tenant_id.ok_or(StatusCode::FORBIDDEN)?;
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

    let rows = query.fetch_all(app.db_pool()).await.map_err(|e| {
        error!("Database error listing short links: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
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

/// Create a short link (skeleton, no persistence yet).
pub async fn create_link(
    State(app): State<CrudApp>,
    auth: ApiKeyAuth,
    Json(req): Json<CreateLinkRequest>,
) -> Result<(StatusCode, Json<LinkResponse>), StatusCode> {
    let tenant_id = auth.tenant_id.ok_or(StatusCode::FORBIDDEN)?;
    let repo = app.short_link_repository();
    let strategy: Box<dyn LinkCreationStrategy> = match req.slug.as_deref().map(str::trim) {
        Some(slug) if !slug.is_empty() => Box::new(ProvidedSlugStrategy::new(slug.to_string())),
        _ => Box::new(GeneratedSlugStrategy::new(6)),
    };

    let strategy_label = strategy.label();
    info!(
        "Create link using strategy={} target_url={} tenant_id={}",
        strategy_label, req.target_url, tenant_id
    );

    let result = strategy
        .create(&repo, &req.target_url, tenant_id)
        .await
        .map_err(|e| match e {
            AppError::Conflict(msg) => {
                warn!(
                    "Slug conflict for tenant {:?} while creating/updating short link: {}",
                    tenant_id, msg
                );
                StatusCode::CONFLICT
            }
            other => {
                error!(
                    "Database error inserting/updating short link via {}: {}",
                    strategy_label, other
                );
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let response = LinkResponse {
        slug: result.0,
        target_url: result.1,
        active: result.2,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct StubRepo {
        upsert_calls: Arc<Mutex<Vec<(String, String, Uuid)>>>,
        generated_calls: Arc<Mutex<Vec<(String, usize, Uuid)>>>,
    }

    #[async_trait]
    impl LinkRepository for StubRepo {
        async fn upsert(
            &self,
            slug: &str,
            target_url: &str,
            tenant_id: Uuid,
        ) -> LinkCreationResult {
            self.upsert_calls.lock().unwrap().push((
                slug.to_string(),
                target_url.to_string(),
                tenant_id,
            ));
            Ok((slug.to_string(), target_url.to_string(), true))
        }

        async fn create_with_generated_slug(
            &self,
            target_url: &str,
            min_len: usize,
            tenant_id: Uuid,
        ) -> LinkCreationResult {
            self.generated_calls
                .lock()
                .unwrap()
                .push((target_url.to_string(), min_len, tenant_id));
            Ok(("generated".into(), target_url.to_string(), true))
        }
    }

    #[tokio::test]
    async fn provided_slug_strategy_uses_upsert() {
        let repo = StubRepo::default();
        let strategy = ProvidedSlugStrategy::new("custom".into());
        let tenant_id = Uuid::nil();

        let result = strategy
            .create(&repo, "https://example.com", tenant_id)
            .await
            .expect("strategy should succeed");

        assert_eq!(result.0, "custom");
        assert_eq!(repo.upsert_calls.lock().unwrap().len(), 1);
        assert!(repo.generated_calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn generated_slug_strategy_requests_generation() {
        let repo = StubRepo::default();
        let strategy = GeneratedSlugStrategy::new(8);
        let tenant_id = Uuid::nil();

        let result = strategy
            .create(&repo, "https://example.com", tenant_id)
            .await
            .expect("strategy should succeed");

        assert_eq!(result.0, "generated");
        assert_eq!(repo.generated_calls.lock().unwrap().len(), 1);
        assert!(repo.upsert_calls.lock().unwrap().is_empty());
    }
}
