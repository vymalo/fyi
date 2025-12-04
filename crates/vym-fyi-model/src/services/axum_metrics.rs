use axum::{
    body::Body,
    extract::connect_info::ConnectInfo,
    http::{Request, Response},
    middleware::Next,
};
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use axum_prometheus::{GenericMetricLayer, Handle, PrometheusMetricLayerBuilder};
use metrics::counter;
use std::net::SocketAddr;

pub const DEFAULT_IGNORE_PATHS: &[&str] = &["/health", "/metrics"];

/// Build a Prometheus metric layer and handle with common ignore patterns.
pub fn prometheus_layer_with_ignores<'a>(
    ignore_patterns: &'a [&'a str],
) -> (
    GenericMetricLayer<'a, PrometheusHandle, Handle>,
    PrometheusHandle,
) {
    PrometheusMetricLayerBuilder::new()
        .with_ignore_patterns(ignore_patterns)
        .with_default_metrics()
        .build_pair()
}

/// Build a Prometheus metric layer/handle with default ignore patterns.
pub fn prometheus_layer_default<'a>() -> (
    GenericMetricLayer<'a, PrometheusHandle, Handle>,
    PrometheusHandle,
) {
    prometheus_layer_with_ignores(DEFAULT_IGNORE_PATHS)
}

/// Middleware that records per-IP request counters.
pub async fn record_ip_metrics(req: Request<Body>, next: Next) -> Response<Body> {
    let method = req.method().as_str().to_owned();
    let path = req.uri().path().to_owned();
    if should_ignore(&path) {
        return next.run(req).await;
    }
    let client_ip = extract_ip(&req);

    let response = next.run(req).await;
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method),
        ("path", path),
        ("status", status),
        ("client_ip", client_ip),
    ];
    counter!("http_requests_by_ip_total", &labels).increment(1);

    response
}

fn should_ignore(path: &str) -> bool {
    DEFAULT_IGNORE_PATHS.iter().any(|p| path.starts_with(p))
}

fn extract_ip<B>(req: &Request<B>) -> String {
    if let Some(fwd) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        && let Some(first) = fwd.split(',').next()
    {
        let ip = first.trim();
        if !ip.is_empty() {
            return ip.to_string();
        }
    }

    if let Some(ConnectInfo(addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
        return addr.ip().to_string();
    }

    "unknown".to_string()
}
