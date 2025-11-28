"use client";

import type { FormEvent } from "react";
import { useRouter } from "next/navigation";
import { Button, Card, CardContent, CardHeader, CardTitle } from "@vymalo/ui";
import { useAuth } from "@vymalo/auth";

function LoginForm() {
  const { client, session } = useAuth();
  const router = useRouter();

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const email = String(formData.get("email") ?? "");
    const nextSession = await client.signIn(email);

    if (nextSession) {
      router.push("/");
    }
  }

  return (
    <form className="space-y-4" onSubmit={handleSubmit}>
      <label className="form-control w-full">
        <div className="label">
          <span className="label-text">Email</span>
        </div>
        <input
          name="email"
          type="email"
          defaultValue={session?.user.email}
          placeholder="you@example.com"
          className="input input-bordered w-full"
          required
        />
      </label>
      <Button className="w-full" type="submit">
        Continue
      </Button>
    </form>
  );
}

export default function LoginPage() {
  return (
    <div className="mx-auto flex max-w-xl flex-col gap-6">
      <Card className="border-base-200">
        <CardHeader>
          <CardTitle>Sign in to continue</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-base-content/70">
            Auth pages demonstrate routing and Better Auth client wiring. Provide any email to bootstrap a demo session.
          </p>
          <LoginForm />
        </CardContent>
      </Card>
      <p className="text-sm text-base-content/70">
        Looking for sign-up? Visit <a href="/register" className="link">create an account</a>.
      </p>
    </div>
  );
}
