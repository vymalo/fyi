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
    title="CRUD API Overview",
    description="Overview of the CRUD /api/links service backed by Postgres.",
    tags=["axum", "crud", "vymalo"],
    timezone="browser",
    time=Time("now-6h", "now"),
    templating=Templating(
        list=[
            Template(
                name="status",
                label="Status",
                dataSource=PROM_DS,
                query='label_values(axum_http_requests_total{job="crud"}, status)',
                includeAll=True,
                allValue=".*",
                multi=False,
            ),
            Template(
                name="client_ip",
                label="Client IP",
                dataSource=PROM_DS,
                query='label_values(http_requests_by_ip_total{job="crud"}, client_ip)',
                includeAll=True,
                allValue=".*",
                multi=False,
            ),
        ]
    ),
    panels=[
        # Overall CRUD traffic, split by endpoint + method.
        TimeSeries(
            title="CRUD Request Rate (RPS)",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "sum by (endpoint, method) ("
                        '  rate(axum_http_requests_total{job="crud",status=~"$status"}[5m])'
                        ")"
                    ),
                    legendFormat="{{method}} {{endpoint}}",
                    refId="A",
                ),
            ],
            unit=OPS_FORMAT,
            gridPos=GridPos(h=8, w=12, x=0, y=0),
        ),
        # /api/links latency from the histogram, p50/p95/p99.
        TimeSeries(
            title="/api/links Latency (p50 / p95 / p99)",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        'histogram_quantile(0.50, '
                        '  sum by (le, method) ('
                        '    rate(axum_http_requests_duration_seconds_bucket'
                        '{job="crud",endpoint="/api/links",status=~"$status"}[5m])'
                        "  ))"
                    ),
                    legendFormat="p50 {{method}}",
                    refId="A",
                ),
                Target(
                    expr=(
                        'histogram_quantile(0.95, '
                        '  sum by (le, method) ('
                        '    rate(axum_http_requests_duration_seconds_bucket'
                        '{job="crud",endpoint="/api/links",status=~"$status"}[5m])'
                        "  ))"
                    ),
                    legendFormat="p95 {{method}}",
                    refId="B",
                ),
                Target(
                    expr=(
                        'histogram_quantile(0.99, '
                        '  sum by (le, method) ('
                        '    rate(axum_http_requests_duration_seconds_bucket'
                        '{job="crud",endpoint="/api/links",status=~"$status"}[5m])'
                        "  ))"
                    ),
                    legendFormat="p99 {{method}}",
                    refId="C",
                ),
            ],
            unit="s",
            gridPos=GridPos(h=8, w=12, x=12, y=0),
        ),
        # Average latency by path using the summary.
        Table(
            title="Average Latency by Path",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(10, "
                        "  sum by (path) (rate(http_request_duration_seconds_sum{job=\"crud\",status=~\"$status\"}[5m]))"
                        "  / "
                        "  sum by (path) (rate(http_request_duration_seconds_count{job=\"crud\",status=~\"$status\"}[5m]))"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=8, w=12, x=0, y=8),
        ),
        # Average response size by path from the summary.
        Table(
            title="Average Response Size by Path",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(10, "
                        "  sum by (path) (rate(http_response_size_bytes_sum{job=\"crud\",status=~\"$status\"}[5m]))"
                        "  / "
                        "  sum by (path) (rate(http_response_size_bytes_count{job=\"crud\",status=~\"$status\"}[5m]))"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=8, w=12, x=12, y=8),
        ),
        # Top clients calling the CRUD API.
        Table(
            title="Top CRUD Clients (IP / UA)",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(10, "
                        "  sum by (client_ip, user_agent, method, path, status) ("
                        "    rate(http_requests_by_ip_total{job=\"crud\",client_ip=~\"$client_ip\",status=~\"$status\"}[5m])"
                        "  )"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=8, w=12, x=0, y=16),
        ),
        # Cache behaviour for CRUD endpoints.
        Table(
            title="CRUD Cache Status by Path",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "sum by (path, cache_status) ("
                        "  rate(http_cache_status_total{job=\"crud\",status=~\"$status\"}[5m])"
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
