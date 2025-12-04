use rocket::State;
use rocket::response::Redirect;
use tracing::{error, info};

use vym_fyi_model::services::repos::ShortLinkRepository;

use crate::RedirectApp;

/// Redirect endpoint skeleton.
///
/// For now this uses a simple table `short_links` with `slug` as the
/// primary key. Slugs are assumed to be globally unique.
#[get("/<slug>")]
pub async fn redirect_short_link(slug: String, app: &State<RedirectApp>) -> Option<Redirect> {
    info!("Redirect requested: slug={}", slug);

    let repo: ShortLinkRepository = app.short_link_repository();
    let result = repo.resolve(&slug).await;

    match result {
        Ok(Some(target)) => {
            info!("Redirecting slug={} to {}", slug, target);
            Some(Redirect::temporary(target))
        }
        Ok(None) => {
            info!("No active short link found for slug={}", slug);
            None
        }
        Err(e) => {
            error!("Database error while resolving slug {}: {}", slug, e);
            None
        }
    }
}
