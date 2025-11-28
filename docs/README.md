# Developer Guide

Entry point for working on this repo. For architecture, see `docs/arc42.md`.

## Stack quick view
- Monorepo via Turborepo + pnpm.
- Apps: Next.js dashboard (`apps/web`), NestJS API (`apps/api`), Rust redirect (`apps/redirect`), db-migrator (`apps/db-migrator`).
- Data: Postgres (Neon in prod) + Redis cache.
- Auth: Better Auth (sessions + built-in API keys) shared via `packages/auth`.
- Schema: Prisma is the source of truth in `packages/db`; Drizzle and SQLx consume it.

## Prerequisites
- Node.js 20+ and pnpm 8+.
- Rust stable (via rustup) for the redirect service.
- Docker Desktop/Engine for local Postgres and Redis.
- Optional: `just`/`make` if you add helper recipes.

## Initial setup
1) Install JS deps: `pnpm install`
2) Install Rust toolchain: `rustup default stable`
3) Prepare environment files (example below) at the repo root for shared values and per-app overrides if needed.

### Example `.env.local`
```
# Postgres/Redis
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/vymalo
REDIS_URL=redis://localhost:6379

# Better Auth
BETTER_AUTH_SECRET=dev-super-secret
BETTER_AUTH_DATABASE_URL=${DATABASE_URL}

# API keys issued by Better Auth (machine access)
# You can seed one manually in the DB while scaffolding the auth package.
REDIRECT_API_KEY=replace-me
```

## Local services
Use Docker Compose (example) to bring up Postgres and Redis:
```
version: "3.9"
services:
  db:
    image: postgres:16
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: vymalo
    ports:
      - "5432:5432"
  redis:
    image: redis:7
    ports:
      - "6379:6379"
```
Run with `docker compose up -d` in the repo root (save as `docker-compose.yml`).

## Database and migrations
- Prisma schema lives in `packages/db/prisma/schema.prisma`.
- Apply migrations locally (after Postgres is up):
  - `pnpm --filter db prisma migrate dev`
- Generate Prisma client and any Drizzle artifacts as you add them:
  - `pnpm --filter db prisma generate`
- SQLx: sync query metadata or docs from the same schema once the Rust app is in place.

## Running apps (expected once apps are scaffolded)
- Next.js dashboard: `pnpm --filter web dev`
- NestJS API: `pnpm --filter api start:dev`
- Rust redirect: `cd apps/redirect && cargo run`
- db-migrator image: build with `pnpm --filter db-migrator build` (or your chosen builder) and run as an initContainer in K8s.

> Note: command names may change depending on how you scaffold the packages; align package.json and Cargo.toml scripts accordingly.

## Lint, format, test (wire these scripts as you scaffold)
- Lint: `pnpm lint`
- Type-check: `pnpm typecheck`
- Tests: `pnpm test`
- Rust: `cargo fmt`, `cargo clippy`, `cargo test` in `apps/redirect`.

## Auth specifics (Better Auth)
- Shared config in `packages/auth`; reuse it in Next.js and NestJS.
- Sessions stored in Postgres via the Better Auth adapter.
- API keys: use Better Auth's built-in API key support; bind keys to user/project rows and avoid custom hashing.
- For local dev, seed a test user and API key directly in Postgres or via a seeding script.

## API and SDK contract
- Expose OpenAPI from the NestJS service (apps/api) once routes are defined.
- Generate the Node SDK from that spec into `packages/sdk-node` (e.g., with `openapi-typescript` or `openapi-generator-cli`).
- Share the generated client with the dashboard via pnpm workspace.

## CI/CD expectations
- Run lint/typecheck/tests for JS/TS packages and fmt/clippy/test for Rust in CI.
- Build Docker images for web/api/redirect/db-migrator as multi-arch (linux/amd64, linux/arm64); use `cross` for Rust binaries and Buildx for Node apps.
- Scan all built images with Trivy before publishing.
- Publish container images to GHCR and Helm charts (OCI) to GHCR; publish npm workspace packages (e.g., SDK/UI/types) to npm.
- Run Prisma migrations in a db-migrator step (initContainer or pre-deploy hook) before rolling out services.

## Where to go next
- Architecture details: `docs/arc42.md`.
- Add ADRs under `docs/adr/` if decisions evolve.
