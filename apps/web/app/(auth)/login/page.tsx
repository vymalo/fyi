"use client";

import type { FormEvent } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { Button, Card, CardContent, CardHeader, CardTitle, Input } from "@vymalo/ui";
import { useAuth } from "@vymalo/auth";

function LoginForm() {
  const { client, session } = useAuth();
  const router = useRouter();

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const email = String(formData.get("email") ?? "");
    const nextSession = await client.signIn.email({ email, password: "demo-password" });

    if (nextSession) {
      router.push("/");
    }
  }

  return (
    <form className="space-y-4" onSubmit={handleSubmit}>
      <div className="space-y-2">
        <label className="text-sm font-medium text-foreground" htmlFor="email">
          Email
        </label>
        <Input
          id="email"
          name="email"
          type="email"
          defaultValue={session?.user.email}
          placeholder="you@example.com"
          required
        />
      </div>
      <Button className="w-full" type="submit">
        Continue
      </Button>
    </form>
  );
}

export default function LoginPage() {
  return (
    <div className="mx-auto flex max-w-xl flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle>Sign in to continue</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4 text-muted-foreground">
          <p className="text-sm">
            Auth pages demonstrate routing and Better Auth client wiring. Provide any email to bootstrap a demo session.
          </p>
          <LoginForm />
        </CardContent>
      </Card>
      <p className="text-sm text-muted-foreground">
        Looking for sign-up? Visit
        {" "}
        <Link href="/register" className="text-primary underline-offset-4 hover:underline">
          create an account
        </Link>
        .
      </p>
    </div>
  );
}
