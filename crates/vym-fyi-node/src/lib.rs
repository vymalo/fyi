use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::services::http_client::HttpClient;

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
    let client = HttpClient::new_with_defaults()?;
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
    let client = HttpClient::new_with_defaults()?;
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
    let client = HttpClient::new_with_defaults()?;
    let base = opts.base_url.trim_end_matches('/');
    let url = format!("{}/api/links", base);

    let mut query_params: Vec<(&str, String)> = Vec::new();

    if let Some(page) = input.page {
        query_params.push(("page", page.to_string()));
    }

    if let Some(per_page) = input.per_page {
        query_params.push(("per_page", per_page.to_string()));
    }

    if let Some(slug) = &input.slug
        && !slug.trim().is_empty()
    {
        query_params.push(("slug", slug.clone()));
    }

    if let Some(target_contains) = &input.target_contains
        && !target_contains.trim().is_empty()
    {
        query_params.push(("target_contains", target_contains.clone()));
    }

    if let Some(active) = input.active {
        query_params.push(("active", active.to_string()));
    }

    if let Some(created_before) = &input.created_before
        && !created_before.trim().is_empty()
    {
        query_params.push(("created_before", created_before.clone()));
    }

    if let Some(created_after) = &input.created_after
        && !created_after.trim().is_empty()
    {
        query_params.push(("created_after", created_after.clone()));
    }

    if let Some(expires_before) = &input.expires_before
        && !expires_before.trim().is_empty()
    {
        query_params.push(("expires_before", expires_before.clone()));
    }

    if let Some(expires_after) = &input.expires_after
        && !expires_after.trim().is_empty()
    {
        query_params.push(("expires_after", expires_after.clone()));
    }

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
