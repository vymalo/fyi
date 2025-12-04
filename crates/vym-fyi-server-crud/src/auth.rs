use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use crate::app::{ApiKeyBinding, CrudApp};
use vym_fyi_model::models::url_shortener::Role;

/// Extracted information about the caller based on their API key.
#[derive(Clone)]
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKeyAuth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(app) = req.rocket().state::<CrudApp>() else {
            return Outcome::Error((Status::InternalServerError, ()));
        };

        let client_id = req.headers().get_one("X-Client-Id");

        let api_key = req.headers().get_one("X-API-Key").or_else(|| {
            req.headers()
                .get_one("Authorization")
                .and_then(|h| h.strip_prefix("ApiKey "))
        });

        let Some(api_key) = api_key else {
            return Outcome::Error((Status::Unauthorized, ()));
        };

        match find_binding(&app.api_keys, api_key, client_id) {
            Some(binding) => Outcome::Success(ApiKeyAuth {
                client_id: binding.client_id.clone(),
                tenant_id: binding.tenant_id,
                role: binding.role.clone(),
                is_master: binding.is_master,
            }),
            None => Outcome::Error((Status::Forbidden, ())),
        }
    }
}
