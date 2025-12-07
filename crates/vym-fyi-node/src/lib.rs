use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::services::http_client::HttpClient;
use vym_fyi_model::services::query_adapter::{LinkListQueryAdapter, QueryParamsBuilder};

#[napi(object)]
pub struct CrudOptions {
    pub base_url: String,
    pub client_id: String,
    pub api_key: String,
    pub master_api_key: Option<String>,
}

impl CrudOptions {
    fn api_key(&self, use_master: bool) -> &str {
        if use_master {
            self.master_api_key.as_deref().unwrap_or(&self.api_key)
        } else {
            &self.api_key
        }
    }
}

#[napi(object)]
pub struct CreateLinkInput {
    pub slug: Option<String>,
    pub target_url: String,
    pub use_master: Option<bool>,
}

#[napi(object)]
pub struct ListLinksInput {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub slug: Option<String>,
    pub target_contains: Option<String>,
    pub active: Option<bool>,
    pub created_before: Option<String>,
    pub created_after: Option<String>,
    pub expires_before: Option<String>,
    pub expires_after: Option<String>,
    pub use_master: Option<bool>,
}

impl LinkListQueryAdapter for ListLinksInput {
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

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkResponse {
    pub slug: String,
    pub target_url: String,
    pub active: bool,
}

#[napi]
pub async fn ping(options: CrudOptions, use_master: Option<bool>) -> Result<()> {
    perform_ping(&options, use_master.unwrap_or(false))
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn create_link(options: CrudOptions, input: CreateLinkInput) -> Result<LinkResponse> {
    let use_master = input.use_master.unwrap_or(false);
    perform_create_link(&options, &input, use_master)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn list_links(options: CrudOptions, input: ListLinksInput) -> Result<Vec<LinkResponse>> {
    let use_master = input.use_master.unwrap_or(false);
    perform_list_links(&options, &input, use_master)
        .await
        .map_err(to_napi_err)
}

async fn perform_ping(opts: &CrudOptions, use_master: bool) -> AppResult<()> {
    let client = HttpClient::global()?;
    let url = format!("{}/health", opts.base_url.trim_end_matches('/'));

    client
        .client()
        .get(&url)
        .header("X-API-Key", opts.api_key(use_master))
        .header("X-Client-Id", &opts.client_id)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

async fn perform_create_link(
    opts: &CrudOptions,
    input: &CreateLinkInput,
    use_master: bool,
) -> AppResult<LinkResponse> {
    let client = HttpClient::global()?;
    let url = format!("{}/api/links", opts.base_url.trim_end_matches('/'));

    let mut body = serde_json::json!({
        "target_url": input.target_url,
    });

    if let Some(slug) = &input.slug {
        body["slug"] = serde_json::Value::String(slug.clone());
    }

    let response = client
        .client()
        .post(&url)
        .header("X-API-Key", opts.api_key(use_master))
        .header("X-Client-Id", &opts.client_id)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    Ok(response.json::<LinkResponse>().await?)
}

async fn perform_list_links(
    opts: &CrudOptions,
    input: &ListLinksInput,
    use_master: bool,
) -> AppResult<Vec<LinkResponse>> {
    let client = HttpClient::global()?;
    let base = opts.base_url.trim_end_matches('/');
    let url = format!("{}/api/links", base);

    let query_params = input.to_query_params();

    let request = client
        .client()
        .get(&url)
        .header("X-API-Key", opts.api_key(use_master))
        .header("X-Client-Id", &opts.client_id)
        .query(&query_params);

    let response = request.send().await?.error_for_status()?;
    Ok(response.json::<Vec<LinkResponse>>().await?)
}

fn to_napi_err<E: std::fmt::Display>(err: E) -> Error {
    Error::from_reason(err.to_string())
}
