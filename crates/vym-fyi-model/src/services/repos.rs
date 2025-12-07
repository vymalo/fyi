use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

use crate::models::errors::{AppError, AppResult};

/// Repository for tenant-related database operations.
#[derive(Clone)]
pub struct TenantRepository {
    pool: Pool<Postgres>,
}

impl TenantRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// List all tenants as (id, name).
    pub async fn list_all(&self) -> AppResult<Vec<(Uuid, String)>> {
        let rows = sqlx::query("SELECT id, name FROM tenants")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let id: Uuid = row.get("id");
                let name: String = row.get("name");
                (id, name)
            })
            .collect())
    }

    /// Create a new tenant with the given name and return its id.
    pub async fn create(&self, name: &str) -> AppResult<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO tenants (id, name, status)
            VALUES ($1, $2, 'active')
            "#,
        )
        .bind(id)
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    /// Delete a tenant by name.
    pub async fn delete_by_name(&self, name: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM tenants WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// Repository for short-link operations.
#[derive(Clone)]
pub struct ShortLinkRepository {
    pool: Pool<Postgres>,
}

impl ShortLinkRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Upsert a short link by slug, target_url, and tenant_id.
    /// Returns (slug, target_url, is_active).
    pub async fn upsert(
        &self,
        slug: &str,
        target_url: &str,
        tenant_id: Uuid,
    ) -> AppResult<(String, String, bool)> {
        let row = sqlx::query(
            r#"
            INSERT INTO short_links (slug, target_url, is_active, tenant_id)
            VALUES ($1, $2, TRUE, $3)
            ON CONFLICT (slug) DO UPDATE
                SET target_url = EXCLUDED.target_url,
                    is_active = TRUE
                WHERE short_links.tenant_id = EXCLUDED.tenant_id
            RETURNING slug, target_url, is_active
            "#,
        )
        .bind(slug)
        .bind(target_url)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok((
                row.get("slug"),
                row.get("target_url"),
                row.get::<bool, _>("is_active"),
            ))
        } else {
            Err(AppError::Conflict(
                "slug already exists for a different tenant".into(),
            ))
        }
    }

    /// List short links for a single tenant as (slug, target_url, is_active)
    /// with pagination support.
    pub async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<(String, String, bool)>> {
        let rows = sqlx::query(
            r#"
            SELECT slug, target_url, is_active
            FROM short_links
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.get("slug"),
                    row.get("target_url"),
                    row.get::<bool, _>("is_active"),
                )
            })
            .collect())
    }

    /// List all short links (any tenant) with pagination.
    pub async fn list_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<(String, String, bool)>> {
        let rows = sqlx::query(
            r#"
            SELECT slug, target_url, is_active
            FROM short_links
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.get("slug"),
                    row.get("target_url"),
                    row.get::<bool, _>("is_active"),
                )
            })
            .collect())
    }

    /// Resolve a slug to a target URL, if any.
    pub async fn resolve(&self, slug: &str) -> AppResult<Option<String>> {
        let row = sqlx::query(
            r#"
            SELECT target_url
            FROM short_links
            WHERE slug = $1
              AND is_active = TRUE
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.get("target_url")))
    }

    /// Create a short link with a generated slug (at least `min_len` characters).
    /// On rare collisions, this will retry a few times before failing.
    pub async fn create_with_generated_slug(
        &self,
        target_url: &str,
        min_len: usize,
        tenant_id: Uuid,
    ) -> AppResult<(String, String, bool)> {
        const MAX_ATTEMPTS: usize = 5;
        for _ in 0..MAX_ATTEMPTS {
            let slug = crate::services::slug::generate_slug(min_len);
            let row = sqlx::query(
                r#"
                INSERT INTO short_links (slug, target_url, is_active, tenant_id)
                VALUES ($1, $2, TRUE, $3)
                ON CONFLICT (slug) DO NOTHING
                RETURNING slug, target_url, is_active
                "#,
            )
            .bind(&slug)
            .bind(target_url)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(row) = row {
                return Ok((
                    row.get("slug"),
                    row.get("target_url"),
                    row.get::<bool, _>("is_active"),
                ));
            }
        }

        Err(AppError::Config(
            "Failed to generate unique slug after several attempts".into(),
        ))
    }
}

/// Abstract factory for repositories.
pub trait RepositoryFactory: Send + Sync {
    fn tenant_repo(&self) -> TenantRepository;
    fn short_link_repo(&self) -> ShortLinkRepository;
}

/// Concrete factory for Postgres-backed repositories.
#[derive(Clone)]
pub struct PgRepositoryFactory {
    pool: Pool<Postgres>,
}

impl PgRepositoryFactory {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

impl RepositoryFactory for PgRepositoryFactory {
    fn tenant_repo(&self) -> TenantRepository {
        TenantRepository::new(self.pool.clone())
    }

    fn short_link_repo(&self) -> ShortLinkRepository {
        ShortLinkRepository::new(self.pool.clone())
    }
}
