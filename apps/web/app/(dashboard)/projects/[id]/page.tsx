import type { Metadata } from "next";
import Link from "next/link";
import { Badge, Button, Card, CardContent, CardHeader, CardTitle } from "@vymalo/ui";
import { DashboardCard, PageHeader } from "../../../../components/app-shell";
import { getApiBaseUrl } from "../../../../lib/env";

export const dynamicParams = true;

export async function generateMetadata({ params }: { params: { id: string } }): Promise<Metadata> {
  return {
    title: `Project ${params.id}`,
  };
}

export default function ProjectDetailPage({ params }: { params: { id: string } }) {
  const apiBase = getApiBaseUrl();

  return (
    <div className="space-y-6">
      <PageHeader
        title={`Project ${params.id}`}
        description={`Ready to fetch redirects from ${apiBase}/projects/${params.id}/links`}
        cta={
          <Button asChild size="sm" variant="outline">
            <Link href="/projects">Back to projects</Link>
          </Button>
        }
      />
      <DashboardCard title="Recent redirects">
        <Card>
          <CardHeader className="flex items-center justify-between">
            <CardTitle>Link roll-up</CardTitle>
            <Badge variant="outline">Sample data</Badge>
          </CardHeader>
          <CardContent className="space-y-2 text-sm text-muted-foreground">
            <p>Use TanStack Query here to hydrate redirects for this project.</p>
            <p>API base URL is pulled from NEXT_PUBLIC_API_BASE_URL.</p>
          </CardContent>
        </Card>
      </DashboardCard>
    </div>
  );
}
