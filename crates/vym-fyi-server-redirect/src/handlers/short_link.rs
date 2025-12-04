use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
};
use tracing::{error, info};

use vym_fyi_model::services::repos::ShortLinkRepository;

use crate::RedirectApp;

/// Redirect endpoint skeleton.
///
/// For now this uses a simple table `short_links` with `slug` as the
/// primary key. Slugs are assumed to be globally unique.
pub async fn redirect_short_link(
    Path(slug): Path<String>,
    State(app): State<RedirectApp>,
) -> Result<Redirect, StatusCode> {
    info!("Redirect requested: slug={}", slug);
    let slug_counter = metrics::counter!("redirect_slug_requests_total", "slug" => slug.clone());
    slug_counter.increment(1);

    let repo: ShortLinkRepository = app.short_link_repository();
    let result = repo.resolve(&slug).await;

    match result {
        Ok(Some(target)) => {
            info!("Redirecting slug={} to {}", slug, target);
            Ok(Redirect::temporary(&target))
        }
        Ok(None) => {
            info!("No active short link found for slug={}", slug);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Database error while resolving slug {}: {}", slug, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
