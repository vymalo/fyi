import Link from "next/link";
import { Badge, Button, Card, CardContent, CardHeader, CardTitle } from "@vymalo/ui";
import { DashboardCard, PageHeader } from "../components/app-shell";

export default function HomePage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Welcome to the workspace dashboard"
        description="Next.js App Router starter wired with TanStack Query, Better Auth, Tailwind CSS, and shadcn/ui primitives."
        cta={
          <Button asChild>
            <Link href="/projects">View projects</Link>
          </Button>
        }
      />

      <div className="grid gap-4 md:grid-cols-3">
        <Card className="md:col-span-2">
          <CardHeader className="mb-2 flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <Badge>Workspace-aware setup</Badge>
              <span>Shared foundation</span>
            </CardTitle>
            <Badge variant="secondary">Turborepo</Badge>
          </CardHeader>
            <CardContent className="space-y-2 text-muted-foreground">
              <p>
                The dashboard consumes shared configuration from <code>packages/config</code> and UI primitives from
                <code>packages/ui</code> to keep apps consistent.
              </p>
            <ul className="list-disc space-y-1 pl-4">
              <li>App Router, TypeScript, Tailwind CSS, and shadcn/ui building blocks.</li>
              <li>TanStack Query and Better Auth client provider wiring.</li>
              <li>Ready-to-use routes for auth and project/link management pages.</li>
            </ul>
          </CardContent>
        </Card>
        <DashboardCard title="Env aware">
          <p>Reads API base URL, Better Auth secret, and API key from <code>.env.local</code>.</p>
          <p className="text-sm text-muted-foreground">See <code>apps/web/.env.local.example</code> to get started.</p>
        </DashboardCard>
      </div>
    </div>
  );
}
