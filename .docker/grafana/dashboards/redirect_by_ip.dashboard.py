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
    title="Redirect – By Client IP",
    description="IP-focused view of redirect traffic, errors, and cache behaviour.",
    tags=["axum", "redirect", "ip", "vymalo"],
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
        ]
    ),
    panels=[
        # Overall redirect RPS by client IP.
        TimeSeries(
            title="Redirect RPS by Client IP",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "sum by (client_ip) ("
                        "  rate(http_requests_by_ip_total{job=\"redirect\",client_ip=~\"$client_ip\"}[5m])"
                        ")"
                    ),
                    legendFormat="{{client_ip}}",
                    refId="A",
                ),
            ],
            unit=OPS_FORMAT,
            gridPos=GridPos(h=8, w=12, x=0, y=0),
        ),
        # Error RPS by client IP.
        TimeSeries(
            title="Error RPS by Client IP",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "sum by (client_ip) ("
                        "  rate(http_requests_by_ip_total{job=\"redirect\",status=~\"4..|5..\",client_ip=~\"$client_ip\"}[5m])"
                        ")"
                    ),
                    legendFormat="{{client_ip}}",
                    refId="A",
                ),
            ],
            unit=OPS_FORMAT,
            gridPos=GridPos(h=8, w=12, x=12, y=0),
        ),
        # Top IPs by total traffic (snapshot).
        Table(
            title="Top Client IPs by Traffic",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(20, "
                        "  sum by (client_ip) ("
                        "    rate(http_requests_by_ip_total{job=\"redirect\",client_ip=~\"$client_ip\"}[5m])"
                        "  )"
                        ")"
                    ),
                    format="table",
                    instant=True,
                    refId="A",
                ),
            ],
            gridPos=GridPos(h=8, w=12, x=0, y=8),
        ),
        # Top IPs by error traffic (snapshot).
        Table(
            title="Top Client IPs by Errors",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "topk(20, "
                        "  sum by (client_ip) ("
                        "    rate(http_requests_by_ip_total{job=\"redirect\",status=~\"4..|5..\",client_ip=~\"$client_ip\"}[5m])"
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
        # Per-IP status class breakdown (4xx vs 5xx vs others).
        Table(
            title="Status Breakdown per Client IP",
            dataSource=PROM_DS,
            targets=[
                Target(
                    expr=(
                        "sum by (client_ip, status) ("
                        "  rate(http_requests_by_ip_total{job=\"redirect\",client_ip=~\"$client_ip\"}[5m])"
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
