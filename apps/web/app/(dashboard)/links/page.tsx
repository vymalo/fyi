"use client";

import { useQuery } from "@tanstack/react-query";
import { Button, Card, CardContent, CardHeader, CardTitle } from "@vymalo/ui";
import { DashboardCard, PageHeader } from "../../../components/app-shell";
import { getApiBaseUrl } from "../../../lib/env";

const sampleLinks = [
  { id: "l-101", project: "p-001", url: "https://example.com/docs", clicks: 128 },
  { id: "l-102", project: "p-002", url: "https://example.com/signup", clicks: 88 },
  { id: "l-103", project: "p-003", url: "https://example.com/changelog", clicks: 34 },
];

export default function LinksPage() {
  const { data } = useQuery({
    queryKey: ["links"],
    queryFn: async () => {
      console.info(`Hydrating links from ${getApiBaseUrl()}/links`);
      return sampleLinks;
    },
  });

  return (
    <div className="space-y-6">
      <PageHeader
        title="Links"
        description="Manage redirect destinations and keep projects organized."
        cta={
          <Button className="btn-sm" variant="secondary">
            New link
          </Button>
        }
      />
      <DashboardCard title="Recent links">
        <div className="grid gap-3 md:grid-cols-2">
          {data?.map((link) => (
            <Card key={link.id} className="border-base-200">
              <CardHeader className="flex items-center justify-between">
                <CardTitle>{link.url}</CardTitle>
                <span className="badge badge-outline">{link.clicks} clicks</span>
              </CardHeader>
              <CardContent className="text-sm text-base-content/70">
                <p>Project: {link.project}</p>
                <p>Link ID: {link.id}</p>
              </CardContent>
            </Card>
          ))}
        </div>
      </DashboardCard>
    </div>
  );
}
