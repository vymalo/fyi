import type { ReactNode } from "react";
import { Card, CardContent } from "@vymalo/ui";

export function PageHeader({
  title,
  description,
  cta,
}: {
  title: string;
  description?: string;
  cta?: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
      <div>
        <h1 className="text-2xl font-bold leading-tight md:text-3xl">{title}</h1>
        {description ? <p className="text-base-content/70">{description}</p> : null}
      </div>
      {cta}
    </div>
  );
}

export function DashboardCard({ title, children }: { title: string; children: ReactNode }) {
  return (
    <Card className="border-base-200 bg-base-100">
      <CardContent>
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold">{title}</h3>
        </div>
        <div className="mt-3 space-y-2 text-sm text-base-content/80">{children}</div>
      </CardContent>
    </Card>
  );
}
