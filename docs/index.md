# vym.fyi

vymalo for your information is a small, multi-tenant URL shortener built in Rust. Use this site as the entry point for working with the codebase and understanding how the services fit together.

## Docs at a glance

- [Developer Guide](README.md): local setup, running the services, and CLI usage.
- [Architecture (Arc42)](arc42.md): system context, design decisions, and deployment model.

## Running the docs locally

1. Install the doc tooling (once):
   ```bash
   pip install mkdocs-material
   ```
2. Serve with live reload from the repo root:
   ```bash
   mkdocs serve
   ```
3. Build the static site (optional):
   ```bash
   mkdocs build
   ```

## Design patterns in the app layer

- Strategy: `ProvidedSlugStrategy` and `GeneratedSlugStrategy` choose how slugs are created in `crates/vym-fyi-server-crud/src/handlers/links.rs`.
- Facade: `CrudApp` and `RedirectApp` wrap the DB pool, repository factory, and API-key state (`crates/vym-fyi-server-crud/src/app.rs`, `crates/vym-fyi-server-redirect/src/app.rs`).
- Factory: `PgRepositoryFactory` implements the `RepositoryFactory` interface to provide database-backed repositories (`crates/vym-fyi-model/src/services/repos.rs`).
- Adapter: `LinkListQueryAdapter` with `QueryParamsBuilder` converts list inputs into HTTP query params for both the CLI and Node bindings (`crates/vym-fyi-model/src/services/query_adapter.rs`).
- Singleton: `HttpClient::global` exposes a shared HTTP client for reuse across crates (`crates/vym-fyi-model/src/services/http_client.rs`).

## Tests and coverage

- Unit tests live alongside services (config placeholder resolution, query adapter, HTTP client singleton, link strategies) and run with `cargo test --workspace --all-targets`.
- Integration tests validate configuration loading and env resolution at `crates/vym-fyi-model/tests/config_flow.rs`.
- CI runs the test suite on every push/PR via `.github/workflows/ci.yml`.
- To measure coverage locally, run `cargo llvm-cov --workspace --all-features --fail-under-lines 70` (70%+ recommended) and publish the generated report if needed.
