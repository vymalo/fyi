"use client";

import Link from "next/link";
import { useQuery } from "@tanstack/react-query";
import { Button, Card, CardContent, CardHeader, CardTitle } from "@vymalo/ui";
import { DashboardCard, PageHeader } from "../../../components/app-shell";
import { getApiBaseUrl } from "../../../lib/env";

const sampleProjects = [
  { id: "p-001", name: "Marketing", linkCount: 3 },
  { id: "p-002", name: "Product", linkCount: 5 },
  { id: "p-003", name: "Developer Relations", linkCount: 2 },
];

export default function ProjectsPage() {
  const { data, isLoading } = useQuery({
    queryKey: ["projects"],
    queryFn: async () => {
      // This is where the dashboard will call the API; we surface the base URL so it is easy to swap when wiring the backend.
      const apiBase = getApiBaseUrl();
      console.info(`Fetching projects from ${apiBase}/projects`);
      return sampleProjects;
    },
  });

  return (
    <div className="space-y-6">
      <PageHeader
        title="Projects"
        description="Manage your workspaces and related redirect links."
        cta={
          <Button className="btn-sm" variant="secondary">
            Create project
          </Button>
        }
      />
      <DashboardCard title="Active projects">
        {isLoading ? (
          <span className="loading loading-dots loading-md" aria-label="Loading projects" />
        ) : (
          <div className="grid gap-4 md:grid-cols-3">
            {data?.map((project) => (
              <Card key={project.id} className="border-base-200">
                <CardHeader className="flex items-center justify-between">
                  <CardTitle>{project.name}</CardTitle>
                  <span className="badge badge-outline">{project.linkCount} links</span>
                </CardHeader>
                <CardContent className="space-y-2">
                  <p className="text-sm text-base-content/70">Project ID: {project.id}</p>
                  <Link href={`/projects/${project.id}`} className="link link-primary text-sm">
                    Manage redirects
                  </Link>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </DashboardCard>
    </div>
  );
}
