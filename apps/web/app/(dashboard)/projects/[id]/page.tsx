import type { Metadata } from "next";
import Link from "next/link";
import { Button, Card, CardContent, CardHeader, CardTitle } from "@vymalo/ui";
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
          <Button variant="outline" className="btn-sm">
            <Link href="/projects">Back to projects</Link>
          </Button>
        }
      />
      <DashboardCard title="Recent redirects">
        <Card className="border-base-200">
          <CardHeader className="flex items-center justify-between">
            <CardTitle>Link roll-up</CardTitle>
            <span className="badge badge-outline">Sample data</span>
          </CardHeader>
          <CardContent className="space-y-2 text-sm text-base-content/80">
            <p>Use TanStack Query here to hydrate redirects for this project.</p>
            <p className="text-base-content/70">API base URL is pulled from NEXT_PUBLIC_API_BASE_URL.</p>
          </CardContent>
        </Card>
      </DashboardCard>
    </div>
  );
}
