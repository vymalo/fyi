use axum::{
    extract::{Path, State},
    http::{HeaderValue, header::CACHE_CONTROL},
    response::{IntoResponse, Redirect, Response},
};
use tracing::{debug, error};

use vym_fyi_model::services::repos::ShortLinkRepository;
use vym_fyi_model::services::static_assets;

use crate::RedirectApp;

/// Redirect endpoint skeleton.
///
/// For now this uses a simple table `short_links` with `slug` as the
/// primary key. Slugs are assumed to be globally unique.
pub async fn redirect_short_link(
    Path(slug): Path<String>,
    State(app): State<RedirectApp>,
) -> Response {
    debug!("Redirect requested: slug={}", slug);
    let slug_counter = metrics::counter!("redirect_slug_requests_total", "slug" => slug.clone());
    slug_counter.increment(1);

    let repo: ShortLinkRepository = app.short_link_repository();
    let result = repo.resolve(&slug).await;

    match result {
        Ok(Some(target)) => {
            debug!("Redirecting slug={} to {}", slug, target);
            let mut response = Redirect::temporary(&target).into_response();
            response.headers_mut().insert(
                CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=60"),
            );
            response
        }
        Ok(None) => {
            debug!("No active short link found for slug={}", slug);
            let mut response = static_assets::not_found().await;
            response
                .headers_mut()
                .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
            response
        }
        Err(e) => {
            error!("Database error while resolving slug {}: {}", slug, e);
            let mut response = static_assets::internal_error().await;
            response
                .headers_mut()
                .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
            response
        }
    }
}
