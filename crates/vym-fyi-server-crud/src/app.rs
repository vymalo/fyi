use std::collections::HashSet;
use std::path::Path;

use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tracing::{info, warn};
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::models::url_shortener::ClientConfig;
use vym_fyi_model::services::config::load_client_config;
use vym_fyi_model::services::repos::{
    PgRepositoryFactory, RepositoryFactory, ShortLinkRepository, TenantRepository,
};

/// Facade over core CRUD server components (DB pool, repositories).
#[derive(Clone)]
pub struct CrudApp {
    pub _pool: Pool<Postgres>,
    pub repos: PgRepositoryFactory,
}

/// Builder for `CrudApp` (builder pattern).
pub struct CrudAppBuilder {
    database_url: String,
    max_connections: u32,
    tenants_config_path: Option<String>,
}

impl CrudAppBuilder {
    /// Construct a builder using standard environment variables.
    pub fn from_env() -> AppResult<Self> {
        let database_url = std::env::var("DATABASE_URL").map_err(|_| {
            vym_fyi_model::models::errors::AppError::Config("DATABASE_URL not set".into())
        })?;
        let tenants_config_path = std::env::var("TENANTS_CONFIG_PATH").ok();

        Ok(Self {
            database_url,
            max_connections: 5,
            tenants_config_path,
        })
    }

    /// Override the maximum number of DB connections.
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Build the `CrudApp` by creating a pool, running migrations,
    /// and synchronizing tenants from the config file (if configured).
    pub async fn build(self) -> AppResult<CrudApp> {
        let pool: Pool<Postgres> = PgPoolOptions::new()
            .max_connections(self.max_connections)
            .connect(&self.database_url)
            .await?;

        // Run embedded migrations (crates/vym-fyi-server-crud/migrations).
        sqlx::migrate!().run(&pool).await?;

        let repos = PgRepositoryFactory::new(pool.clone());
        let app = CrudApp { _pool: pool, repos };

        if let Some(path) = self.tenants_config_path {
            app.sync_tenants_from_config(Path::new(&path)).await?;
        } else {
            warn!("TENANTS_CONFIG_PATH not set; skipping tenant synchronization");
        }

        Ok(app)
    }
}

impl CrudApp {
    pub fn tenant_repository(&self) -> TenantRepository {
        self.repos.tenant_repo()
    }

    pub fn short_link_repository(&self) -> ShortLinkRepository {
        self.repos.short_link_repo()
    }

    /// Synchronize tenants based on the provided YAML config.
    ///
    /// This delegates to the tenant repository for all persistence.
    pub async fn sync_tenants_from_config(&self, path: &Path) -> AppResult<()> {
        let config: ClientConfig = load_client_config(path)?;

        let desired: HashSet<String> = config.clients.keys().cloned().collect();
        info!("tenant sync: {} tenants defined in config", desired.len());

        let repo = self.tenant_repository();
        let existing = repo.list_all().await?;
        let existing_names: HashSet<String> =
            existing.iter().map(|(_, name)| name.clone()).collect();

        // Create tenants in config but missing in DB.
        for tenant_name in desired.difference(&existing_names) {
            info!("tenant sync: creating tenant name={}", tenant_name);
            let _ = repo.create(tenant_name).await?;
        }

        // Delete tenants that exist in DB but not in config.
        for (_, name) in existing
            .into_iter()
            .filter(|(_, name)| !desired.contains(name))
        {
            info!("tenant sync: deleting tenant name={}", name);
            repo.delete_by_name(&name).await?;
        }

        Ok(())
    }
}
