use crate::app::{ApiKeyBinding, CrudApp};
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use std::future::ready;
use tracing::info;
use vym_fyi_model::models::url_shortener::Role;

/// Extracted information about the caller based on their API key.
#[derive(Clone, Debug)]
pub struct ApiKeyAuth {
    #[allow(dead_code)]
    pub client_id: Option<String>,
    pub tenant_id: Option<uuid::Uuid>,

    #[allow(dead_code)]
    pub role: Option<Role>,
    pub is_master: bool,
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let max = ab.len().max(bb.len());
    let mut diff: u8 = (ab.len() ^ bb.len()) as u8;
    for i in 0..max {
        let av = *ab.get(i).unwrap_or(&0);
        let bv = *bb.get(i).unwrap_or(&0);
        diff |= av ^ bv;
    }
    diff == 0
}

fn find_binding<'a>(
    bindings: &'a [ApiKeyBinding],
    key: &str,
    client_id: Option<&str>,
) -> Option<&'a ApiKeyBinding> {
    // First, check for a matching master key (master keys are global).
    let master = bindings
        .iter()
        .find(|b| b.is_master && constant_time_eq(&b.key, key));
    if master.is_some() {
        return master;
    }

    // For non-master keys, require a matching client id if one is provided.
    if let Some(cid) = client_id {
        bindings.iter().find(|b| {
            !b.is_master && b.client_id.as_deref() == Some(cid) && constant_time_eq(&b.key, key)
        })
    } else {
        // No client id and not a master key: reject.
        None
    }
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
            Some(api_key) => match find_binding(&api_keys, &api_key, client_id.as_deref()) {
                Some(binding) => Ok(ApiKeyAuth {
                    client_id: binding.client_id.clone(),
                    tenant_id: binding.tenant_id,
                    role: binding.role.clone(),
                    is_master: binding.is_master,
                }),
                None => Err(StatusCode::FORBIDDEN),
            },
            None => Err(StatusCode::UNAUTHORIZED),
        })
    }
}
