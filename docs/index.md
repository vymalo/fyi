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
