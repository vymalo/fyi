import Link from "next/link";
import { Badge, Button, Card, CardContent, CardHeader, CardTitle } from "@vymalo/ui";
import { PageHeader } from "../../../components/app-shell";

export default function RegisterPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Create a workspace account"
        description="Auth routes are ready to connect to the Better Auth API."
        cta={
          <Button asChild variant="outline">
            <Link href="/login">Back to login</Link>
          </Button>
        }
      />
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Badge variant="secondary">Preview</Badge>
            <span>Sign up is in progress</span>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 text-sm text-muted-foreground">
          <p>
            Wire this page to your Better Auth instance and use the shared <code>@vymalo/auth</code> client to submit data to the
            API base URL configured in <code>.env.local</code>.
          </p>
          <p>
            The layout, buttons, and cards come from <code>@vymalo/ui</code> so you can quickly build consistent experiences across
            apps.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
