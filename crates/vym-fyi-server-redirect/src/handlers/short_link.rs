use rocket::response::Redirect;
use tracing::info;

/// Redirect endpoint skeleton.
///
/// For now this only logs the request and returns 404 (no redirect).
/// A later iteration will look up the slug in Postgres using the
/// tenant id and issue an HTTP redirect.
#[get("/<tenant>/<slug>")]
pub async fn redirect_short_link(tenant: String, slug: String) -> Option<Redirect> {
    info!("Redirect requested: tenant={} slug={}", tenant, slug);
    None
}
