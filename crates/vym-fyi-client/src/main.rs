#[macro_use]
extern crate log;

use crate::shared::cli::{Command, Opt};
use crate::shared::config::{ResolvedClient, load_client_config, resolve_client};
use clap::Parser;
use env_logger::{Builder, Env};
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::services::http_client::HttpClient;
use vym_fyi_model::services::query_adapter::{LinkListQueryAdapter, QueryParamsBuilder};

pub mod shared;

/// Parameters for the `links-list` CLI command.
#[derive(Debug)]
struct LinksListParams {
    page: Option<u32>,
    per_page: Option<u32>,
    slug: Option<String>,
    target_contains: Option<String>,
    active: Option<bool>,
    created_before: Option<String>,
    created_after: Option<String>,
    expires_before: Option<String>,
    expires_after: Option<String>,
}

impl LinkListQueryAdapter for LinksListParams {
    fn to_query_params(&self) -> Vec<(&'static str, String)> {
        let mut builder = QueryParamsBuilder::new();
        builder
            .push_value("page", self.page)
            .push_value("per_page", self.per_page)
            .push_trimmed("slug", &self.slug)
            .push_trimmed("target_contains", &self.target_contains)
            .push_value("active", self.active)
            .push_trimmed("created_before", &self.created_before)
            .push_trimmed("created_after", &self.created_after)
            .push_trimmed("expires_before", &self.expires_before)
            .push_trimmed("expires_after", &self.expires_after);
        builder.into_vec()
    }
}

/// Entry point: configures logging and runs the app workflow.
#[tokio::main]
async fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("starting up");

    match app().await {
        Ok(_) => {
            info!("Done!");
        }
        Err(e) => {
            error!("An error occurred during execution: {}", e);
        }
    }
}

async fn app() -> AppResult<()> {
    let opt = Opt::try_parse()?;

    let config = load_client_config(&opt.config)?;
    let resolved = resolve_client(&config, &opt.client)?;

    match opt.command {
        Command::Ping => ping(&resolved, opt.use_master).await,
        Command::LinksCreate { slug, target } => {
            links_create(&resolved, opt.use_master, slug, target).await
        }
        Command::LinksList {
            page,
            per_page,
            slug,
            target_contains,
            active,
            created_before,
            created_after,
            expires_before,
            expires_after,
        } => {
            links_list(
                &resolved,
                opt.use_master,
                LinksListParams {
                    page,
                    per_page,
                    slug,
                    target_contains,
                    active,
                    created_before,
                    created_after,
                    expires_before,
                    expires_after,
                },
            )
            .await
        }
    }
}

fn select_api_key(client: &ResolvedClient, use_master: bool) -> &str {
    if use_master {
        client
            .master_api_key
            .as_deref()
            .unwrap_or(&client.entry.api_key)
    } else {
        &client.entry.api_key
    }
}

async fn ping(client: &ResolvedClient, use_master: bool) -> AppResult<()> {
    let http = HttpClient::global()?;
    let url = format!("{}/health", client.base_url.trim_end_matches('/'));

    info!("Pinging CRUD server at {}", url);

    let api_key = select_api_key(client, use_master);

    let response = http
        .client()
        .get(&url)
        .header("X-API-Key", api_key)
        .header("X-Client-Id", &client.id)
        .send()
        .await?;
    info!("Received status: {}", response.status());

    Ok(())
}

async fn links_create(
    client: &ResolvedClient,
    use_master: bool,
    slug: Option<String>,
    target: String,
) -> AppResult<()> {
    let http = HttpClient::global()?;
    let url = format!("{}/api/links", client.base_url.trim_end_matches('/'));

    match &slug {
        Some(s) => info!("Creating link slug={} target={}", s, target),
        None => info!("Creating link with generated slug target={}", target),
    }

    let api_key = select_api_key(client, use_master);

    let body = if let Some(slug) = slug {
        serde_json::json!({
            "slug": slug,
            "target_url": target,
        })
    } else {
        serde_json::json!({
            "target_url": target,
        })
    };

    let response = http
        .client()
        .post(&url)
        .header("X-API-Key", api_key)
        .header("X-Client-Id", &client.id)
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    info!("Server responded with {}: {}", status, text);

    Ok(())
}

async fn links_list(
    client: &ResolvedClient,
    use_master: bool,
    params: LinksListParams,
) -> AppResult<()> {
    let http = HttpClient::global()?;
    let base = client.base_url.trim_end_matches('/');
    let url = format!("{}/api/links", base);

    info!("Listing links");

    let api_key = select_api_key(client, use_master);

    let mut req = http
        .client()
        .get(&url)
        .header("X-API-Key", api_key)
        .header("X-Client-Id", &client.id);

    let query_params = params.to_query_params();
    if !query_params.is_empty() {
        req = req.query(&query_params);
    }

    let response = req.send().await?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    info!("Server responded with {}", status);
    println!("{text}");

    Ok(())
}
