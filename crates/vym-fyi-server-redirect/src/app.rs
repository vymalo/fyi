use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::services::repos::{PgRepositoryFactory, RepositoryFactory, ShortLinkRepository};

/// Facade over redirect server components.
#[derive(Clone)]
pub struct RedirectApp {
    repos: PgRepositoryFactory,
}

/// Builder for `RedirectApp`.
pub struct RedirectAppBuilder {
    database_url_ro: String,
    max_connections: u32,
}

impl RedirectAppBuilder {
    pub fn from_env() -> AppResult<Self> {
        let database_url_ro = std::env::var("DATABASE_URL_RO").map_err(|_| {
            vym_fyi_model::models::errors::AppError::Config("DATABASE_URL_RO not set".into())
        })?;

        Ok(Self {
            database_url_ro,
            max_connections: 5,
        })
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub async fn build(self) -> AppResult<RedirectApp> {
        let pool: Pool<Postgres> = PgPoolOptions::new()
            .max_connections(self.max_connections)
            .connect(&self.database_url_ro)
            .await?;

        let repos = PgRepositoryFactory::new(pool);

        Ok(RedirectApp { repos })
    }
}

impl RedirectApp {
    pub fn short_link_repository(&self) -> ShortLinkRepository {
        self.repos.short_link_repo()
    }
}
