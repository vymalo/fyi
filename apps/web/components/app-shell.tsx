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
        <h1 className="text-3xl font-semibold leading-tight">{title}</h1>
        {description ? <p className="text-muted-foreground">{description}</p> : null}
      </div>
      {cta}
    </div>
  );
}

export function DashboardCard({ title, children }: { title: string; children: ReactNode }) {
  return (
    <Card>
      <CardContent className="pt-6">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold">{title}</h3>
        </div>
        <div className="mt-3 space-y-2 text-sm text-muted-foreground">{children}</div>
      </CardContent>
    </Card>
  );
}
