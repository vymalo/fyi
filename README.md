# vym.fyi – tiny, API‑only URL shortener (Rust)

`vym.fyi` is a minimal, multi‑tenant URL shortener implemented in Rust. It ships two small services:

- `vym-fyi-server-crud`: API for creating/listing short links (multi‑tenant, API key protected).
- `vym-fyi-server-redirect`: latency‑focused redirector that resolves slugs and issues redirects.

There is intentionally **no dashboard**—only an HTTP API and CLI—so the footprint stays tiny and no resources are burned on UI hosting or rendering.

## Why this exists

- **Tiny images**: redirect image is ~6.5 MB (`ghcr.io/vymalo/fyi-redirect:chore-dual-appa20ea015dbf8a805c5d1191f4ce6f22dd8e9c58d`).
- **Cloud native**: ships as two containers with health checks, metrics, and OTLP hooks; ready for K8s/Compose.
- **Multi‑tenant by design**: API keys are scoped to tenants; master key optional.
- **Fast path**: redirect service is read‑only and keeps the hot path lean (Axum + Rust async).
- **DRY patterns**: shared repository factory, app facades, link‑creation strategies, HTTP query adapters, and a singleton HTTP client.

## Quick start

Requirements: Rust stable, Docker (optional), Postgres.

```bash
# run tests
cargo test --workspace --all-targets

# run CRUD API locally (env: DATABASE_URL, TENANTS_CONFIG_PATH optional)
cargo run -p vym-fyi-server-crud

# run redirect server locally (env: DATABASE_URL_RO)
cargo run -p vym-fyi-server-redirect
```

Using Docker Compose (uses the multi‑stage Dockerfile):

```bash
docker compose up --build
```

## Usage flow (API only)

1. Prepare a tenant config (`config.yaml`) with API keys; env placeholders like `$(CLIENT_A)` are resolved at runtime.
2. Start the CRUD API with that config; it syncs tenants and API key bindings.
3. Call the API (or use the CLI) to create/list links.
4. Point traffic to the redirect service for `/{slug}`.

## Comparison with other self‑hosted shorteners

Image sizes measured via `docker manifest inspect` for amd64; features sourced from project docs.

| Project | Stack | Dashboard/UI | API | Cloud‑native story | Image size (amd64) | DB deps | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **vym.fyi (this)** | Rust (Axum) | None (deliberate) | Yes (CRUD API + CLI) | Containers, health, Prometheus `/metrics`, OTLP | ~6.5 MB (redirect) | Postgres | Multi‑tenant keys; read/write vs read‑only roles |
| Shlink ([shlink.io](https://shlink.io/)) | PHP + RoadRunner | Optional web client | Yes | Docker/K8s friendly, but heavier runtime | ~102 MB (`shlinkio/shlink:latest`) | Postgres/MySQL | Has web UI; broader feature set (tags, visits) |
| Kutt ([github.com/thedevs-network/kutt](https://github.com/thedevs-network/kutt)) | Node.js | Yes | Yes | Docker image provided | ~109 MB (`kutt/kutt:latest`) | Postgres, Redis | Modern UI, requires Redis |
| YOURLS ([yourls.org](https://yourls.org/)) | PHP | Yes (admin UI) | Plugins / limited API | Docker images exist | ~186 MB (`yourls:latest`) | MySQL | Mature, plugin ecosystem |

## Architecture at a glance

- Two binaries: CRUD API (read/write DB) and Redirect (read‑only DB).
- API key auth only; `X-API-Key` or `Authorization: ApiKey …`.
- Prometheus `/metrics` and OTLP tracing/metrics exposed by both services.
- Patterns: Facade (`CrudApp`, `RedirectApp`), Factory (`RepositoryFactory` + `PgRepositoryFactory`), Strategy (`ProvidedSlugStrategy` vs `GeneratedSlugStrategy`), Adapter (`LinkListQueryAdapter`), Singleton (`HttpClient::global`).

## Testing & coverage

- Run everything: `cargo test --workspace --all-targets`.
- Coverage (recommended ≥70%): `cargo llvm-cov --workspace --all-features --fail-under-lines 70`.
- CI (`.github/workflows/ci.yml`) runs lint/format/test on every push/PR.

## License

MIT, see `LICENSE`.
