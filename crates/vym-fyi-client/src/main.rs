#[macro_use]
extern crate log;

use crate::shared::cli::{Command, Opt};
use crate::shared::config::{ResolvedClient, load_client_config, resolve_client};
use clap::Parser;
use env_logger::{Builder, Env};
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::services::http_client::HttpClient;

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
    let http = HttpClient::new_with_defaults()?;
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
    let http = HttpClient::new_with_defaults()?;
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
    let http = HttpClient::new_with_defaults()?;
    let base = client.base_url.trim_end_matches('/');
    let url = format!("{}/api/links", base);

    info!("Listing links");

    let api_key = select_api_key(client, use_master);

    let mut req = http
        .client()
        .get(&url)
        .header("X-API-Key", api_key)
        .header("X-Client-Id", &client.id);

    let mut query_params: Vec<(&str, String)> = Vec::new();

    if let Some(p) = params.page {
        query_params.push(("page", p.to_string()));
    }

    if let Some(pp) = params.per_page {
        query_params.push(("per_page", pp.to_string()));
    }

    if let Some(s) = params.slug
        && !s.trim().is_empty()
    {
        query_params.push(("slug", s));
    }

    if let Some(t) = params.target_contains
        && !t.trim().is_empty()
    {
        query_params.push(("target_contains", t));
    }

    if let Some(a) = params.active {
        query_params.push(("active", a.to_string()));
    }

    if let Some(cb) = params.created_before
        && !cb.trim().is_empty()
    {
        query_params.push(("created_before", cb));
    }

    if let Some(ca) = params.created_after
        && !ca.trim().is_empty()
    {
        query_params.push(("created_after", ca));
    }

    if let Some(eb) = params.expires_before
        && !eb.trim().is_empty()
    {
        query_params.push(("expires_before", eb));
    }

    if let Some(ea) = params.expires_after
        && !ea.trim().is_empty()
    {
        query_params.push(("expires_after", ea));
    }

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
