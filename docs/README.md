# Developer Guide

Entry point for working on this repo. For architecture details, see `docs/arc42.md`.

This repository contains a small, multi‑tenant URL shortener implemented as a Rust workspace with multiple binaries:

- `vym-fyi-server-crud`: CRUD/API server for tenants, API keys, and short links.
- `vym-fyi-server-redirect`: redirect server used on the hot path for resolving short URLs.
- `vym-fyi-client`: CLI for calling the CRUD API using API keys from `config.yaml`.
- `vym-fyi-model`: shared models and telemetry utilities.
- `vym-fyi-healthcheck`: lightweight health probe binary for container health checks.

The services are intended to run in containers and on Kubernetes, using the existing `Dockerfile`, `compose.yaml`, and Helm charts under `charts/`.

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

Both servers are Rocket applications; they respect `ROCKET_` environment variables (for example `ROCKET_ADDRESS`, `ROCKET_PORT`) and additional app‑specific environment variables for database URLs and telemetry.

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

A `config.yaml` is structured as follows:

```yaml
server:
  base_url: https://crud.example.com

clients:
  client-a:
    name: my-cli-client-a
    api_key: "$(CLIENT_A_SECRET)"
    role: admin

  client-b:
    name: my-cli-client-b
    api_key: "$(CLIENT_B_SECRET)"
    role: url
```

- `server.base_url`: base URL for the CRUD API.
- `clients`: map of client ids to client configuration; each client corresponds to a tenant.
- `api_key`: may contain placeholders of the form `$(ENV_VAR_NAME)`; the CLI resolves these using environment variables at runtime.

Typical usage:

- `vym-fyi-client ... --client client-a`
  The CLI:
  - Loads `config.yaml`.
  - Selects `clients.client-a`.
  - Resolves `api_key` (substituting any `$(ENV_VAR)` placeholders).
  - Sends requests to `server.base_url` using that API key.

## Metrics and Observability

Both servers are instrumented with OpenTelemetry utilities from `vym-fyi-model`:

- Traces and metrics are exported via OTLP to an OpenTelemetry Collector endpoint.
- Each server exposes a Prometheus‑compatible `/metrics` endpoint for scraping.

In Kubernetes, a typical observability stack is:

- OpenTelemetry Collector (receives OTLP from the services).
- Prometheus (scrapes `/metrics` and/or the collector’s Prometheus exporter).
- Grafana (dashboards for CRUD/Redirect metrics and traces).

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
