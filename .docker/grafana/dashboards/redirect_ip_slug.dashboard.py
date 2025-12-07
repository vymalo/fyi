from grafanalib.core import (
    Dashboard,
    GridPos,
    Table,
    Target,
    Templating,
    Template,
    Time,
)


PROM_DS = "Prometheus"


dashboard = Dashboard(
    title="Redirect – IP × Slug",
    description="Correlation of client IPs and slugs for redirect traffic and errors.",
    tags=["axum", "redirect", "ip", "slug", "vymalo"],
    timezone="browser",
    time=Time("now-6h", "now"),
    templating=Templating(
        list=[
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
                label="Slug (path)",
                dataSource=PROM_DS,
                query='label_values(http_requests_by_ip_total{job="redirect"}, path)',
                includeAll=True,
                allValue=".*",
                multi=False,
            ),
        ]
    ),
    panels=[
        # Top client_ip x path combinations by traffic.
        Table(
            title="Top IP × Path by Traffic",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(50, "
                        "  sum by (client_ip, path, status) ("
                        "    rate(http_requests_by_ip_total{job=\"redirect\",client_ip=~\"$client_ip\",path=~\"$slug\"}[5m])"
                        "  )"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=10, w=24, x=0, y=0),
        ),
        # Top client_ip x path combinations by error traffic.
        Table(
            title="Top IP × Path by Errors",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(50, "
                        "  sum by (client_ip, path, status) ("
                        "    rate(http_requests_by_ip_total{job=\"redirect\",status=~\"4..|5..\",client_ip=~\"$client_ip\",path=~\"$slug\"}[5m])"
                        "  )"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=10, w=24, x=0, y=10),
        ),
    ],
).auto_panel_ids()
