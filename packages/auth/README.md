# Auth Package

Shared Better Auth client wiring for frontend apps.

- Uses `better-auth/client` under the hood to create a React-friendly auth client.
- `AuthProvider` and `useAuth` expose session state through Better Auth's `useSession` hook.
- Secrets and API keys are passed through `AuthClientConfig` and sourced in the web app from `.env.local`.
