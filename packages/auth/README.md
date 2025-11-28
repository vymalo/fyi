# Auth Package

Shared Better Auth client wiring for frontend apps.

- `createAuthClient` builds a fetch-friendly client using workspace env vars.
- `AuthProvider` and `useAuth` expose session state to React components.
- Secrets and API keys are passed through `AuthClientConfig` and sourced in the web app from `.env.local`.
