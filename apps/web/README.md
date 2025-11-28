# Web App

Next.js (App Router) dashboard scaffolded with Tailwind CSS, shadcn/ui primitives, daisyUI themes, TanStack Query, and Better Auth wiring.

## Scripts
- `pnpm --filter web dev`
- `pnpm --filter web lint`
- `pnpm --filter web typecheck`
- `pnpm --filter web build`

## Environment
Copy `.env.local.example` to `.env.local` and set:
- `NEXT_PUBLIC_API_BASE_URL` pointing to the API service.
- `BETTER_AUTH_SECRET` and `NEXT_PUBLIC_BETTER_AUTH_API_KEY` to reach Better Auth.

The app reads shared TypeScript/ESLint presets from `packages/config` and UI components from `packages/ui`.
