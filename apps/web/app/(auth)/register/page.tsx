import Link from "next/link";
import { Button, Card, CardContent, CardHeader, CardTitle } from "@vymalo/ui";
import { PageHeader } from "../../../components/app-shell";

export default function RegisterPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Create a workspace account"
        description="Auth routes are ready to connect to the Better Auth API."
        cta={
          <Button>
            <Link href="/login">Back to login</Link>
          </Button>
        }
      />
      <Card className="border-base-200">
        <CardHeader>
          <CardTitle>Sign up is in progress</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 text-base-content/80">
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
