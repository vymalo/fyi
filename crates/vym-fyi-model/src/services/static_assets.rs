use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use tower_http::services::ServeDir;

/// Directory where static assets (HTML, icons, manifest, …) live.
pub const STATIC_DIR: &str = "static";

/// Attach shared static routes:
/// - `/static/*` served from the `static` directory
/// - `/favicon.ico` permanent redirect to `/static/favicon.ico`
pub fn attach_static_routes<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .nest_service("/static", ServeDir::new(STATIC_DIR))
        .route(
            "/favicon.ico",
            get(|| async { Redirect::permanent("/static/favicon.ico") }),
        )
}

/// Render the shared 404 page as an Axum response.
pub async fn not_found() -> Response {
    error_page("404.html", StatusCode::NOT_FOUND, "404 – Not Found").await
}

/// Render the shared 500 page as an Axum response.
pub async fn internal_error() -> Response {
    error_page(
        "500.html",
        StatusCode::INTERNAL_SERVER_ERROR,
        "500 – Internal Server Error",
    )
    .await
}

async fn error_page(file: &str, status: StatusCode, fallback: &'static str) -> Response {
    let path = format!("{}/{}", STATIC_DIR, file);
    let body = tokio::fs::read_to_string(path)
        .await
        .unwrap_or_else(|_| fallback.to_string());

    (status, Html(body)).into_response()
}
