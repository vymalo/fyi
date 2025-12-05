use axum::{
    body::Body,
    extract::connect_info::ConnectInfo,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use axum_prometheus::{GenericMetricLayer, Handle, PrometheusMetricLayerBuilder};
use metrics::{counter, histogram};
use std::{net::SocketAddr, time::Instant};

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
    let user_agent = extract_user_agent(&req);
    let start = Instant::now();

    let response = next.run(req).await;
    let latency = start.elapsed().as_secs_f64();
    let status_code = response.status();
    let status = status_code.as_u16().to_string();
    let cache_status = extract_cache_status(&response);
    let response_size = extract_response_size(&response);

    let labels = [
        ("method", method),
        ("path", path),
        ("status", status),
        ("client_ip", client_ip),
        ("user_agent", user_agent),
    ];
    counter!("http_requests_by_ip_total", &labels).increment(1);

    let core_labels = [
        ("method", labels[0].1.clone()),
        ("path", labels[1].1.clone()),
        ("status", labels[2].1.clone()),
    ];

    histogram!("http_request_duration_seconds", &core_labels).record(latency);

    if let Some(bytes) = response_size {
        histogram!("http_response_size_bytes", &core_labels).record(bytes as f64);
    }

    if status_code.is_client_error() || status_code.is_server_error() {
        let error_labels = [
            ("method", core_labels[0].1.clone()),
            ("path", core_labels[1].1.clone()),
            ("status", core_labels[2].1.clone()),
            ("class", status_class(status_code).to_string()),
        ];
        counter!("http_request_errors_total", &error_labels).increment(1);
    }

    let cache_labels = [
        ("method", core_labels[0].1.clone()),
        ("path", core_labels[1].1.clone()),
        ("status", core_labels[2].1.clone()),
        ("cache_status", cache_status),
    ];
    counter!("http_cache_status_total", &cache_labels).increment(1);

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

fn extract_user_agent<B>(req: &Request<B>) -> String {
    req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(str::trim)
        .filter(|ua| !ua.is_empty())
        .map(|ua| ua.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn extract_response_size(res: &Response<Body>) -> Option<u64> {
    if let Some(len) = res
        .headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
    {
        return Some(len);
    }

    None
}

fn extract_cache_status<B>(res: &Response<B>) -> String {
    const CANDIDATE_HEADERS: &[&str] = &["x-cache", "x-cache-status", "cf-cache-status"];

    for key in CANDIDATE_HEADERS {
        if let Some(value) = res
            .headers()
            .get(*key)
            .and_then(|h| h.to_str().ok())
            .and_then(|raw| raw.split([' ', ',']).find(|s| !s.is_empty()))
        {
            return value.to_ascii_lowercase();
        }
    }

    if res.headers().contains_key("age") {
        return "hit".to_string();
    }

    "none".to_string()
}

fn status_class(status: StatusCode) -> &'static str {
    match status.as_u16() {
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    }
}
