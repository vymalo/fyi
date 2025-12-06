from grafanalib.core import (
    Dashboard,
    GridPos,
    OPS_FORMAT,
    Table,
    Target,
    Templating,
    Template,
    Time,
    TimeSeries,
)


PROM_DS = "Prometheus"


dashboard = Dashboard(
    title="Redirector Overview",
    description="Overview of the public redirect service and short-link usage.",
    tags=["axum", "redirect", "vymalo"],
    timezone="browser",
    time=Time("now-6h", "now"),
    templating=Templating(
        list=[
            Template(
                name="status",
                label="Status",
                dataSource=PROM_DS,
                query='label_values(axum_http_requests_total{job="redirect"}, status)',
                includeAll=True,
                allValue=".*",
                multi=False,
            ),
            Template(
                name="client_ip",
                label="Client IP",
                dataSource=PROM_DS,
                query='label_values(http_requests_by_ip_total{job="redirect"}, client_ip)',
                includeAll=True,
                allValue=".*",
                multi=False,
            ),
            Template(
                name="slug",
                label="Slug/Path",
                dataSource=PROM_DS,
                query='label_values(http_requests_by_ip_total{job="redirect"}, path)',
                includeAll=True,
                allValue=".*",
                multi=False,
            ),
        ]
    ),
    panels=[
        # Overall redirect traffic, split by endpoint + status.
        TimeSeries(
            title="Redirect Request Rate (RPS)",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "sum by (endpoint, status, method) ("
                        '  rate(axum_http_requests_total{job="redirect",status=~"$status"}[5m])'
                        ")"
                    ),
                    legendFormat="{{method}} {{status}} {{endpoint}}",
                    refId="A",
                ),
            ],
            unit=OPS_FORMAT,
            gridPos=GridPos(h=8, w=12, x=0, y=0),
        ),
        # Error rate for redirect traffic.
        TimeSeries(
            title="Redirect Error Rate (%)",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "100 * "
                        "sum(rate(http_request_errors_total{job=\"redirect\",status=~\"$status\"}[5m]))"
                        " / "
                        "sum(rate(axum_http_requests_total{job=\"redirect\",status=~\"$status\"}[5m]))"
                    ),
                    legendFormat="error rate",
                    refId="A",
                ),
            ],
            unit="percent",
            gridPos=GridPos(h=8, w=12, x=12, y=0),
        ),
        # Slug popularity.
        Table(
            title="Top Slugs by Traffic",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(10, rate(redirect_slug_requests_total{job=\"redirect\"}[5m]))"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=8, w=12, x=0, y=8),
        ),
        # Paths generating errors.
        Table(
            title="Top Error Paths",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(10, "
                        "  sum by (path, status) ("
                        "    rate(http_request_errors_total{job=\"redirect\",status=~\"$status\",path=~\"$slug\"}[5m])"
                        "  )"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=8, w=12, x=12, y=8),
        ),
        # Cache behaviour for redirect endpoints.
        Table(
            title="Redirect Cache Status by Path",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "sum by (path, cache_status) ("
                        "  rate(http_cache_status_total{job=\"redirect\",status=~\"$status\",path=~\"$slug\"}[5m])"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=8, w=12, x=0, y=16),
        ),
        # Clients of the redirector.
        Table(
            title="Top Redirect Clients (IP / UA)",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(10, "
                        "  sum by (client_ip, user_agent, method, path, status) ("
                        "    rate(http_requests_by_ip_total{job=\"redirect\",client_ip=~\"$client_ip\",status=~\"$status\",path=~\"$slug\"}[5m])"
                        "  )"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=8, w=12, x=12, y=16),
        ),
    ],
).auto_panel_ids()
