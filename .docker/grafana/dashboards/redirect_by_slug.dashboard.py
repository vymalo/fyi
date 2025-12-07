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
    title="Redirect – By Slug",
    description="Slug-focused view of redirect traffic, latency, and errors.",
    tags=["axum", "redirect", "slug", "vymalo"],
    timezone="browser",
    time=Time("now-6h", "now"),
    templating=Templating(
        list=[
            Template(
                name="slug",
                label="Slug",
                dataSource=PROM_DS,
                query='label_values(redirect_slug_requests_total{job="redirect"}, slug)',
                includeAll=True,
                allValue=".*",
                multi=False,
            ),
        ]
    ),
    panels=[
        # Overall redirect RPS by slug.
        TimeSeries(
            title="RPS per Slug",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "sum by (slug) ("
                        "  rate(redirect_slug_requests_total{job=\"redirect\",slug=~\"$slug\"}[5m])"
                        ")"
                    ),
                    legendFormat="{{slug}}",
                    refId="A",
                ),
            ],
            unit=OPS_FORMAT,
            gridPos=GridPos(h=8, w=12, x=0, y=0),
        ),
        # Latency by endpoint for slug redirects (404 vs 3xx etc.).
        TimeSeries(
            title="Slug Latency (p95, by status)",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        'histogram_quantile(0.95, '
                        '  sum by (le, status) ('
                        '    rate(axum_http_requests_duration_seconds_bucket'
                        '{job="redirect",endpoint="/{slug}"}[5m])'
                        "  ))"
                    ),
                    legendFormat="p95 {{status}}",
                    refId="A",
                ),
            ],
            unit="s",
            gridPos=GridPos(h=8, w=12, x=12, y=0),
        ),
        # Top slugs by traffic (snapshot).
        Table(
            title="Top Slugs by Traffic",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(20, "
                        "  rate(redirect_slug_requests_total{job=\"redirect\",slug=~\"$slug\"}[5m])"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=8, w=12, x=0, y=8),
        ),
        # Slugs / paths with most errors.
        Table(
            title="Top Error Paths (Slug-like)",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(20, "
                        "  sum by (path, status) ("
                        "    rate(http_request_errors_total{job=\"redirect\"}[5m])"
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
        # Cache behaviour by slug path.
        Table(
            title="Cache Status by Slug Path",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "sum by (path, cache_status) ("
                        "  rate(http_cache_status_total{job=\"redirect\"}[5m])"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=8, w=24, x=0, y=16),
        ),
    ],
).auto_panel_ids()
