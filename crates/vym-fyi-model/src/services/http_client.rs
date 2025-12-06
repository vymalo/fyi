use std::time::Duration;

use crate::models::errors::{AppError, AppResult};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::instrument;

static DEFAULT_HTTP_CLIENT: Lazy<Result<HttpClient, AppError>> =
    Lazy::new(HttpClient::new_with_defaults);

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    #[instrument]
    pub fn new_with_defaults() -> AppResult<Self> {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(16)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60))
            .build()?;
        Ok(Self { client })
    }

    pub fn from_client(client: Client) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Shared HTTP client (Singleton) with the default configuration.
    pub fn global() -> AppResult<&'static HttpClient> {
        match &*DEFAULT_HTTP_CLIENT {
            Ok(client) => Ok(client),
            Err(err) => Err(AppError::Config(format!(
                "failed to initialize shared HTTP client: {}",
                err
            ))),
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn fetch_json<R: DeserializeOwned>(&self, url: &str) -> AppResult<R> {
        let resp = self.client.get(url).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    #[instrument(level = "debug", skip(self, body))]
    pub async fn post_json<B: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> AppResult<R> {
        let resp = self
            .client
            .post(url)
            .json(body)
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    #[instrument(level = "debug", skip(self, token, body))]
    pub async fn post_json_auth<B: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
        body: &B,
    ) -> AppResult<R> {
        let resp = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(body)
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn global_returns_same_instance() {
        let first = HttpClient::global().expect("global client should initialize");
        let second = HttpClient::global().expect("global client should initialize");

        assert!(
            std::ptr::eq(first, second),
            "global client should be a singleton"
        );
    }
}
