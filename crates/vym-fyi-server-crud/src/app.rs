use std::collections::HashSet;
use std::path::Path;

use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tracing::{info, warn};
use uuid::Uuid;
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::models::url_shortener::{ClientConfig, Role};
use vym_fyi_model::services::config::{load_client_config, resolve_env_placeholders};
use vym_fyi_model::services::repos::{
    PgRepositoryFactory, RepositoryFactory, ShortLinkRepository, TenantRepository,
};

/// Simple in-memory representation of an API key binding derived from the tenants config.
#[derive(Clone)]
pub struct ApiKeyBinding {
    pub key: String,
    pub client_id: Option<String>,
    pub tenant_id: Option<Uuid>,
    pub role: Option<Role>,
    pub is_master: bool,
}

/// Facade over core CRUD server components (DB pool, repositories).
#[derive(Clone)]
pub struct CrudApp {
    pub _pool: Pool<Postgres>,
    pub repos: PgRepositoryFactory,
    pub api_keys: Vec<ApiKeyBinding>,
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

        if let Some(path) = self.tenants_config_path {
            let path_buf = Path::new(&path).to_path_buf();
            let config: ClientConfig = load_client_config(&path_buf)?;

            // Synchronize tenants from config.
            {
                let tenant_repo = repos.tenant_repo();
                sync_tenants_with_repo(&tenant_repo, &config).await?;
            }

            // Build mapping tenant_name -> tenant_id.
            let tenant_repo = repos.tenant_repo();
            let existing = tenant_repo.list_all().await?;
            let mut tenant_map = std::collections::HashMap::new();
            for (id, name) in existing {
                tenant_map.insert(name, id);
            }

            // Build in-memory API key bindings from config using tenant ids.
            let api_keys = build_api_key_bindings(&config, &tenant_map)?;

            Ok(CrudApp {
                _pool: pool,
                repos,
                api_keys,
            })
        } else {
            warn!(
                "TENANTS_CONFIG_PATH not set; skipping tenant synchronization and API key bindings"
            );
            Ok(CrudApp {
                _pool: pool,
                repos,
                api_keys: Vec::new(),
            })
        }
    }
}

impl CrudApp {
    #[allow(dead_code)]
    pub fn tenant_repository(&self) -> TenantRepository {
        self.repos.tenant_repo()
    }

    pub fn short_link_repository(&self) -> ShortLinkRepository {
        self.repos.short_link_repo()
    }

    /// Lookup the tenant_id for a given client id, if known.
    #[allow(dead_code)]
    pub fn tenant_id_for_client(&self, client_id: &str) -> Option<Uuid> {
        self.api_keys
            .iter()
            .find(|b| b.client_id.as_deref() == Some(client_id) && b.tenant_id.is_some())
            .and_then(|b| b.tenant_id)
    }
}

async fn sync_tenants_with_repo(repo: &TenantRepository, config: &ClientConfig) -> AppResult<()> {
    let desired: HashSet<String> = config.clients.keys().cloned().collect();
    info!("tenant sync: {} tenants defined in config", desired.len());

    let existing = repo.list_all().await?;
    let existing_names: HashSet<String> = existing.iter().map(|(_, name)| name.clone()).collect();

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

fn build_api_key_bindings(
    config: &ClientConfig,
    tenant_map: &std::collections::HashMap<String, Uuid>,
) -> AppResult<Vec<ApiKeyBinding>> {
    let mut bindings = Vec::new();

    // Per-client API keys
    for (client_id, entry) in &config.clients {
        let resolved_key = resolve_env_placeholders(&entry.api_key)?;
        let tenant_id = tenant_map.get(client_id).copied();
        bindings.push(ApiKeyBinding {
            key: resolved_key,
            client_id: Some(client_id.clone()),
            tenant_id,
            role: entry.role.clone(),
            is_master: false,
        });
    }

    // Optional master API key
    if let Some(raw_master) = &config.server.master_api_key {
        let resolved_master = resolve_env_placeholders(raw_master)?;
        bindings.push(ApiKeyBinding {
            key: resolved_master,
            client_id: None,
            tenant_id: None,
            role: Some(Role::Admin),
            is_master: true,
        });
    }

    Ok(bindings)
}
