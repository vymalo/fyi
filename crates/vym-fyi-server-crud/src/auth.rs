use crate::app::CrudApp;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use std::future::ready;
use tracing::info;

/// Extracted information about the caller based on their API key.
#[derive(Clone, Debug)]
pub struct ApiKeyAuth {
    pub tenant_id: Option<uuid::Uuid>,
    pub is_master: bool,
}

impl FromRequestParts<CrudApp> for ApiKeyAuth {
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut Parts,
        state: &CrudApp,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let client_id = parts
            .headers
            .get("X-Client-Id")
            .and_then(|h| h.to_str().ok())
            .map(str::to_owned);
        info!("auth client_id={:?}", client_id);

        let api_key = parts
            .headers
            .get("X-API-Key")
            .and_then(|h| h.to_str().ok())
            .map(str::to_owned)
            .or_else(|| {
                parts
                    .headers
                    .get("Authorization")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|h| h.strip_prefix("ApiKey "))
                    .map(str::to_owned)
            });

        let api_keys = state.api_keys.clone();

        ready(match api_key {
            Some(api_key) => api_keys
                .authenticate(&api_key, client_id.as_deref())
                .map(|binding| ApiKeyAuth {
                    tenant_id: binding.tenant_id,
                    is_master: binding.is_master,
                })
                .ok_or(StatusCode::FORBIDDEN),
            None => Err(StatusCode::UNAUTHORIZED),
        })
    }
}
