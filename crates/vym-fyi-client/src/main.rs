#[macro_use]
extern crate log;

use crate::shared::cli::{Command, Opt};
use crate::shared::config::{ResolvedClient, load_client_config, resolve_client};
use clap::Parser;
use env_logger::{Builder, Env};
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::services::http_client::HttpClient;

pub mod shared;

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
    }
}

async fn ping(client: &ResolvedClient, use_master: bool) -> AppResult<()> {
    let http = HttpClient::new_with_defaults()?;
    let url = format!("{}/health", client.base_url.trim_end_matches('/'));

    info!("Pinging CRUD server at {}", url);

    let api_key = if use_master {
        client
            .master_api_key
            .as_deref()
            .unwrap_or(&client.entry.api_key)
    } else {
        &client.entry.api_key
    };

    let response = http
        .client()
        .get(&url)
        .header("X-API-Key", api_key)
        .send()
        .await?;
    info!("Received status: {}", response.status());

    Ok(())
}
