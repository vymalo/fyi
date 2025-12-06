# Developer Guide

Entry point for working on this repo. For architecture details, see `docs/arc42.md`.

This repository contains a small, multi‑tenant URL shortener implemented as a Rust workspace with multiple binaries:

- `vym-fyi-server-crud`: CRUD/API server for tenants, API keys, and short links.
- `vym-fyi-server-redirect`: redirect server used on the hot path for resolving short URLs.
- `vym-fyi-client`: CLI for calling the CRUD API using API keys from `config.yaml`.
- `vym-fyi-model`: shared models and telemetry utilities.
- `vym-fyi-healthcheck`: lightweight health probe binary for container health checks.

The services are intended to run in containers and on Kubernetes, using the existing `Dockerfile`, `compose.yaml`, and Helm charts under `charts/`.

## Roadmap

**0–3 months (Foundations)**
- Harden auth and multi‑tenancy: stricter API key scoping, per‑tenant rate limits, and audit logs for CRUD changes.
- Observability: expand Axum metrics (latency histograms, error counters, cache hit/miss), structured logs, and span propagation between CRUD/Redirect.
- Reliability: connection pooling defaults, healthcheck coverage for dependencies, and CI that runs migrations plus smoke tests.

**3–6 months (Scale & UX)**
- Redirect path performance: DB read replicas, optional in‑memory cache for hot slugs, and graceful degradation when Postgres is unavailable.
- CLI/DevEx: richer `vym-fyi-client` commands (bulk import/export, link analytics), autocompletion, and clearer error messages.
- Tenant experience: soft‑delete links with TTL, per‑tenant quotas, and link labels/tags for organization.

**6–12 months (Product polish)**
- Analytics surface: aggregated usage dashboards per tenant (clicks, referrers, status trends), with privacy‑safe retention policies.
- Delivery hardening: blue/green deploy flow in charts, rollout SLOs, and automated rollback triggers tied to metrics.
- Ecosystem: SDK snippets for common languages and templated examples for self‑hosting with Terraform/Helm.

## Prerequisites

- Rust (stable toolchain via `rustup`, edition 2024 in this workspace).
- Docker (for building/running images locally).
- A Postgres instance (local container or managed DB).
- Optional: `kubectl` and Helm for Kubernetes deployments.

## Initial setup

1. Install Rust toolchain:
   - `rustup default stable`
2. Fetch dependencies and build once to warm the cache:
   - `cargo build`
3. Ensure you have a Postgres database available and note:
   - Connection URL for the CRUD server (read/write user).
   - Connection URL for the redirect server (read‑only user).

Exact environment variables for DB configuration, telemetry, and HTTP settings are defined in the server crates and Helm chart values (see the `charts/` directory).

## Running locally

### Using Cargo

You can run each server directly:

- CRUD server:
  - `cargo run -p vym-fyi-server-crud -- <args>`
- Redirect server:
  - `cargo run -p vym-fyi-server-redirect -- <args>`

The healthcheck binary can be run as:

- `cargo run -p vym-fyi-healthcheck`

### Using Docker Compose

The root `compose.yaml` defines services for CRUD and Redirect using the multi‑stage `Dockerfile`:

- `crud` → builds and runs the `vym-fyi-server-crud` target.
- `redirect` → builds and runs the `vym-fyi-server-redirect` target.

You can start them with:

- `docker compose up --build`

You still need to provide a Postgres instance and appropriate environment variables (for example via Docker networking or a managed DB reachable from the containers).

## CLI and `config.yaml`

The `vym-fyi-client` crate provides a CLI that connects to the CRUD server using API keys defined in a YAML configuration file.

You can use the same YAML structure for:
- Local CLI usage (for example `./config/config.yaml`), and
- The tenants file used by the CRUD server in Docker (`.docker/tenants.yaml`).

### Tenant / client configuration file

At minimum, a config file looks like this:

```yaml
server:
  base_url: http://localhost:8000
  master_api_key: "$(MASTER_API_KEY)"  # optional, for admin operations

clients:
  client-a:
    name: client-a
    api_key: "$(CLIENT_A_SECRET)"
    role: admin

  client-b:
    name: client-b
    api_key: "$(CLIENT_B_SECRET)"
    role: url
```

- `server.base_url`: base URL for the CRUD API.
- `server.master_api_key`: optional master API key used for administrative operations from the CLI (also supports `$(ENV_VAR_NAME)` placeholders).
- `clients`: map of client ids to client configuration; each client corresponds to a tenant (the key, e.g. `client-a`, is used as the tenant name when the CRUD server syncs tenants).
- `name`: human-readable name for the client; typically the same as the key.
- `api_key`: may contain placeholders of the form `$(ENV_VAR_NAME)`; the CLI resolves these using environment variables at runtime.
- `role`:
  - `admin`: full management capabilities for that tenant (intended for automation/ops).
  - `url`: limited scope, mainly for creating and managing links.

To use environment placeholders, export the variables before running anything, for example:

```bash
export MASTER_API_KEY="super-admin-key"
export CLIENT_A_SECRET="client-a-key"
export CLIENT_B_SECRET="client-b-key"
```

### How the tenants file is used

- In Docker, the CRUD server uses `.docker/tenants.yaml` (mounted as `/config/tenants.yaml` in `compose.yaml`) as its tenant configuration file.
- On startup, the server:
  - Reads `TENANTS_CONFIG_PATH` (points to `/config/tenants.yaml`).
  - Creates tenants in the database for every client id found under `clients`.
  - Deletes any tenants from the database whose name is no longer present in the file.
- You can think of the tenants file as the source of truth for which tenants exist in the system.

If you add a new client entry to `.docker/tenants.yaml` and restart the CRUD container, a new tenant row will be created automatically.

### CLI usage (step by step, noobs welcome)

1. **Prepare a config file for the CLI**

   You can reuse `.docker/tenants.yaml` or create a dedicated file such as `config/config.yaml` with the same structure as shown above.

2. **Export the environment variables for your keys**

   ```bash
   export MASTER_API_KEY="super-admin-key"
   export CLIENT_A_SECRET="client-a-key"
   export CLIENT_B_SECRET="client-b-key"
   ```

3. **Start the stack with Docker Compose**

   From the repository root:

   ```bash
   docker compose up --build
   ```

   This will start:
   - Postgres (`db`)
   - CRUD server (`crud`)
   - Redirect server (`redirect`)

4. **Ping the CRUD server using a client API key**

   From another terminal, still in the repo root:

   ```bash
   cargo run -p vym-fyi-client -- \
     --config .docker/tenants.yaml \
     --client client-a \
     ping
   ```

   What happens:
   - The CLI loads `.docker/tenants.yaml`.
   - It selects `clients.client-a`.
   - It replaces `$(CLIENT_A_SECRET)` with the value of the `CLIENT_A_SECRET` environment variable.
   - It calls `http://localhost:8000/health` with headers:
     - `X-API-Key: client-a-key`
     - `X-Client-Id: client-a`

5. **Ping using the master API key**

   If `MASTER_API_KEY` is set and `server.master_api_key` is configured:

   ```bash
   cargo run -p vym-fyi-client -- \
     --config .docker/tenants.yaml \
     --client client-a \
     --use-master \
     ping
   ```

   In this case, the CLI:
   - Still reads the same config and client entry.
   - Resolves both `api_key` and `master_api_key` env placeholders.
   - Sends `X-API-Key: super-admin-key` instead of the client-specific key.
   - Still sets `X-Client-Id: client-a`, but the server accepts the master key for any client id.

6. **Create a short link from the CLI**

   Use the `links-create` command to create or update a slug:

   ```bash
   cargo run -p vym-fyi-client -- \
     --config .docker/tenants.yaml \
     --client client-a \
     links-create \
       --slug promo-2025 \
       --target https://example.com/landing
   ```

   What happens:
   - The CLI picks the right API key as before.
   - It sends a `POST` request to `http://localhost:8000/api/links` with JSON:
     - `{ "slug": "promo-2025", "target_url": "https://example.com/landing" }`.
   - The CRUD server stores or updates that short link in the database.

   You can also omit the slug entirely and let the server generate it for you:

   ```bash
   cargo run -p vym-fyi-client -- \
     --config .docker/tenants.yaml \
     --client client-a \
     links-create \
       --target https://example.com/landing
   ```

   In this case:
   - The CLI only sends `{ "target_url": "https://example.com/landing" }`.
   - The CRUD server generates a random, URL-safe slug with at least 6 characters.
   - The response body includes the generated `slug` so you can copy/paste it for use in URLs.

7. **List short links from the CLI**

   Use the `links-list` command to see all known links:

   ```bash
   cargo run -p vym-fyi-client -- \
     --config .docker/tenants.yaml \
     --client client-a \
     links-list
   ```

   The CLI:
   - Calls `GET http://localhost:8000/api/links`.
   - Prints the JSON response to your terminal (an array of objects with `slug`, `target_url`, and `active`).

   You can also narrow down the results using filters and pagination:

   ```bash
   cargo run -p vym-fyi-client -- \
     --config .docker/tenants.yaml \
     --client client-a \
     links-list \
       --page 1 \
       --per-page 50 \
       --slug promo-2025 \
       --target-contains landing \
       --active true \
       --created-after 2025-01-01T00:00:00Z \
       --created-before 2025-12-31T23:59:59Z
   ```

   These flags map directly to HTTP query parameters on `/api/links`:

   - `--page` → `page` (1-based, default 1).
   - `--per-page` → `per_page` (default 20, max 100).
   - `--slug` → `slug` (exact slug match).
   - `--target-contains` → `target_contains` (case-insensitive substring match in `target_url`).
   - `--active` → `active` (`true` / `false`).
   - `--created-after` / `--created-before` → `created_after` / `created_before`
     (RFC3339 timestamps on `created_at`, for example `2025-01-01T00:00:00Z`).
   - `--expires-after` / `--expires-before` → `expires_after` / `expires_before`
     (RFC3339 timestamps on `expires_at`).

All CLI commands follow the same basic pattern:
- You point to a config file with `--config`.
- You choose which client (tenant) to act as with `--client`.
- Optionally you add `--use-master` to use the master API key instead of the client-specific key.

## Metrics and Observability

Both servers are instrumented with OpenTelemetry utilities from `vym-fyi-model`:

- Traces and metrics are exported via OTLP to an OpenTelemetry Collector endpoint.
- Each server exposes a Prometheus‑compatible `/metrics` endpoint for scraping.

In Kubernetes, a typical observability stack is:

- OpenTelemetry Collector (receives OTLP from the services).
- Prometheus (scrapes `/metrics` and/or the collector’s Prometheus exporter).
- Grafana (dashboards for CRUD/Redirect metrics and traces).

### Recommended dashboard metrics

Core metrics (both CRUD and Redirect):

- Request volume and shape: RPS by service, HTTP method, and endpoint using `axum_http_requests_total`.
- Latency percentiles per endpoint: p50/p95/p99 from `axum_http_requests_duration_seconds_*` and `http_request_duration_seconds_*`.
- Error rate and error budget: error percentage from `http_request_errors_total` divided by total requests, broken down by status, class (`4xx`/`5xx`), and endpoint.
- Saturation / load: in‑flight requests from `axum_http_requests_pending`, per service.
- Payload characteristics: average and p95 response size per path using `http_response_size_bytes_*` to detect large or growing payloads.

CRUD API–specific metrics (`job="crud"`):

- Read vs write mix: RPS for `GET /api/links` vs `POST /api/links` to understand usage and capacity needs.
- `/api/links` latency by operation: p95/p99 latency split by method (GET vs POST) from histograms and summaries.
- Success vs client/auth errors: counts of `4xx` by status (e.g. validation vs auth errors) via `http_request_errors_total`.
- Tenant or client usage: if a tenant label is added, RPS and error rate by tenant or API key.
- Database health (via Postgres exporter): connection counts, query latency, slow queries, and cache hit ratio, correlated with `/api/links` latency.

Redirector‑specific metrics (`job="redirect"`):

- Redirect quality: ratio of valid redirects (3xx) vs invalid/expired slugs (4xx/5xx) for `endpoint="/{slug}"`.
- Slug popularity: `redirect_slug_requests_total` to show top slugs by traffic and their trends over time.
- Slug health: slugs or paths with the most errors, combining `redirect_slug_requests_total` and `http_request_errors_total`.
- Abuse / brute‑force detection: IPs with high RPS and high 4xx/404 ratio using `http_requests_by_ip_total` and `http_request_errors_total`.
- Cache / CDN effectiveness: cache hit ratio and status distribution from `http_cache_status_total`, split between static assets and redirect endpoints.
- Bot vs browser traffic: approximate split by `user_agent` (e.g. `curl`/`bot` vs major browsers) using `http_requests_by_ip_total`.

Cross‑cutting “experience” metrics:

- User‑centred latency SLOs: e.g. “99.9% of redirect requests < 50ms and non‑5xx”, “99% of CRUD writes < 250ms and non‑5xx”.
- Availability / uptime: `1 - (5xx requests / total requests)` per service, surfaced as gauges and alerts.
- Rate limiting / throttling (if implemented): counters for requests rejected due to rate limits, per IP or API key.

### Local Grafana dashboards (dashboards‑as‑code)

This repo ships a small observability stack that you can run via Docker Compose:

- `prometheus`: scrapes the CRUD and Redirect `/metrics` endpoints.
- `grafana`: visualises metrics using provisioned dashboards.
- `grafana-dashgen`: a one‑shot “init” container that generates Grafana dashboards from Python using [`grafanalib`](https://github.com/weaveworks/grafanalib).

Dashboard sources live under:

- `./.docker/grafana/dashboards/*.dashboard.py` – grafanalib definitions.
- `./.docker/grafana/provisioning/datasources/prometheus.yml` – Prometheus datasource pointing at `http://prometheus:9090`.
- `./.docker/grafana/provisioning/dashboards/axum.yml` – file‑based dashboard provisioning from `/tmp/grafana_dashboards` inside the Grafana container.

The Compose services tie this together as follows:

- `grafana-dashgen`:
  - Image: `python:3.12-slim`.
  - Mounts:
    - `./.docker/grafana/dashboards` → `/dashboards_src` (Python sources).
    - Named volume `grafana_dashboards` → `/out` (generated JSON).
  - Command:
    - Installs `grafanalib`.
    - Runs `generate-dashboard` for each `*.dashboard.py`:
      - `crud.dashboard.py` → `axum_crud_overview.json`
      - `redirect.dashboard.py` → `axum_redirect_overview.json`
      - `redirect_by_ip.dashboard.py` → `redirect_by_ip.json`
      - `redirect_by_slug.dashboard.py` → `redirect_by_slug.json`
      - `redirect_ip_slug.dashboard.py` → `redirect_ip_slug.json`
    - All JSON files are written to `/out` (backed by the `grafana_dashboards` volume).

- `grafana`:
  - Image: `grafana/grafana`.
  - Mounts:
    - `grafana` → `/var/lib/grafana` (Grafana state).
    - `grafana_dashboards` → `/tmp/grafana_dashboards` (generated dashboards).
    - `./.docker/grafana/provisioning` → `/etc/grafana/provisioning` (datasource + dashboard providers).
  - Depends on `grafana-dashgen` with `condition: service_completed_successfully` so dashboards are generated before Grafana starts.
  - On startup, Grafana picks up:
    - The Prometheus datasource.
    - All JSON dashboards in `/tmp/grafana_dashboards`.

#### Available dashboards and filters

All dashboards are parameterised using Grafana template variables (defined via grafanalib `Templating`/`Template`):

- **CRUD API Overview** (`axum_crud_overview.json`)
  - Focus: `/api/links` behaviour and CRUD clients.
  - Variables:
    - `$status` – HTTP status codes from `axum_http_requests_total{job="crud"}`.
    - `$client_ip` – client IPs from `http_requests_by_ip_total{job="crud"}`.
  - Panels (all respect the variables where it makes sense):
    - Request RPS by endpoint/method filtered by `$status`.
    - `/api/links` latency p50/p95/p99 (histogram) filtered by `$status`.
    - Average latency and response size by path (summaries) filtered by `$status`.
    - Top CRUD clients (IP / user agent) filtered by `$client_ip` and `$status`.
    - CRUD cache status by path filtered by `$status`.

- **Redirector Overview** (`axum_redirect_overview.json`)
  - Focus: overall redirect health and top paths/clients.
  - Variables:
    - `$status` – HTTP status codes from `axum_http_requests_total{job="redirect"}`.
    - `$client_ip` – client IPs from `http_requests_by_ip_total{job="redirect"}`.
    - `$slug` – paths (slug‑like) from `http_requests_by_ip_total{job="redirect"}`.
  - Panels:
    - Redirect RPS by endpoint/method/status filtered by `$status`.
    - Redirect error rate (%) computed from `http_request_errors_total` and `axum_http_requests_total`, filtered by `$status`.
    - Top slugs by traffic using `redirect_slug_requests_total`.
    - Top error paths filtered by `$status` and `$slug`.
    - Redirect cache status by path filtered by `$status` and `$slug`.
    - Top redirect clients (IP / UA) filtered by `$client_ip`, `$status`, and `$slug`.

- **Redirect – By Client IP** (`redirect_by_ip.json`)
  - Focus: traffic and errors per client IP.
  - Variable:
    - `$client_ip` – IPs from `http_requests_by_ip_total{job="redirect"}` (with “All” option).
  - Panels:
    - Time series: redirect RPS by client IP filtered by `$client_ip`.
    - Time series: error RPS by client IP (4xx/5xx) filtered by `$client_ip`.
    - Tables: top IPs by traffic and by errors (top‑K snapshots).
    - Table: status breakdown per IP (`client_ip,status`) filtered by `$client_ip`.

- **Redirect – By Slug** (`redirect_by_slug.json`)
  - Focus: slug popularity and health.
  - Variable:
    - `$slug` – slugs from `redirect_slug_requests_total{job="redirect"}` (with “All” option).
  - Panels:
    - Time series: RPS per slug filtered by `$slug`.
    - Time series: p95 latency by status for `endpoint="/{slug}"`.
    - Table: top slugs by traffic filtered by `$slug`.
    - Table: top error paths (slug‑like paths) from `http_request_errors_total`.
    - Table: cache status by slug path from `http_cache_status_total`.

- **Redirect – IP × Slug** (`redirect_ip_slug.json`)
  - Focus: correlation of client IPs and slugs/paths.
  - Variables:
    - `$client_ip` – IPs from `http_requests_by_ip_total{job="redirect"}`.
    - `$slug` – paths from `http_requests_by_ip_total{job="redirect"}`.
  - Panels (both snapshots):
    - Top `(client_ip, path, status)` combinations by traffic filtered by `$client_ip` and `$slug`.
    - Top `(client_ip, path, status)` combinations by error traffic (4xx/5xx) filtered by `$client_ip` and `$slug`.

#### How to run and use Grafana locally

1. From the repo root, start the full stack:

   ```bash
   docker compose up --build
   ```

   This starts:
   - Postgres (`db`)
   - CRUD API (`crud`)
   - Redirector (`redirect`)
   - Prometheus (`prometheus`)
   - Dashboard generator (`grafana-dashgen`)
   - Grafana (`grafana`)

2. Open Grafana at:

   - `http://localhost:3000` (default login: `admin` / `admin`).

3. Explore dashboards in the “Vymalo” folder:

   - `CRUD API Overview`
   - `Redirector Overview`
   - `Redirect – By Client IP`
   - `Redirect – By Slug`
   - `Redirect – IP × Slug`

4. Use the dashboard variables (`Status`, `Client IP`, `Slug/Path`) to slice metrics per status code, caller IP, or slug without editing any queries.

## Testing, linting, formatting

- Format: `cargo fmt`
- Lint: `cargo clippy --workspace --all-targets`
- Tests: `cargo test --workspace`

These commands should be wired into CI alongside security tooling:

- `deny.toml` for `cargo-deny`.
- `trivy.yaml` for container image scanning.

## Where to go next

- Architecture and design: `docs/arc42.md`.
- Charts and deployment: `charts/` for Helm charts of CRUD and Redirect servers.
- Crate‑specific details: per‑crate `README.md` files under `crates/` (if present).
