import Link from "next/link";
import { Button, Card, CardContent, CardHeader, CardTitle } from "@vymalo/ui";
import { DashboardCard, PageHeader } from "../components/app-shell";

export default function HomePage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Welcome to the workspace dashboard"
        description="Next.js app router starter wired with TanStack Query, Better Auth, Tailwind, shadcn/ui, and daisyUI."
        cta={
          <Button>
            <Link href="/projects">View projects</Link>
          </Button>
        }
      />

      <div className="grid gap-4 md:grid-cols-3">
        <Card className="border-base-200 bg-base-100 md:col-span-2">
          <CardHeader className="mb-2 flex items-center justify-between">
            <CardTitle>Workspace-aware setup</CardTitle>
            <span className="badge badge-primary">Turborepo</span>
          </CardHeader>
          <CardContent className="space-y-2">
            <p>
              The dashboard consumes shared configuration from <code className="badge badge-neutral">packages/config</code>
              and UI primitives from <code className="badge badge-neutral">packages/ui</code> to keep apps consistent.
            </p>
            <ul className="list-disc space-y-1 pl-4">
              <li>App Router, TypeScript, Tailwind CSS, shadcn/ui utilities, and daisyUI themes.</li>
              <li>TanStack Query and Better Auth client provider wiring.</li>
              <li>Ready-to-use routes for auth and project/link management pages.</li>
            </ul>
          </CardContent>
        </Card>
        <DashboardCard title="Env aware">
          <p>Reads API base URL, Better Auth secret, and API key from <code>.env.local</code>.</p>
          <p className="text-sm text-base-content/70">See <code>apps/web/.env.local.example</code> to get started.</p>
        </DashboardCard>
      </div>
    </div>
  );
}
