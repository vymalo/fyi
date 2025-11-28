"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Button, cn } from "@vymalo/ui";
import { useAuth } from "@vymalo/auth";

const links = [
  { href: "/", label: "Overview" },
  { href: "/projects", label: "Projects" },
  { href: "/links", label: "Links" },
  { href: "/login", label: "Auth" },
];

export function Navbar() {
  const pathname = usePathname();
  const { session, loading } = useAuth();

  return (
    <header className="sticky top-0 z-20 border-b bg-background/80 backdrop-blur">
      <div className="container mx-auto flex h-16 items-center justify-between px-4">
        <div className="flex items-center gap-4">
          <Link href="/" className="text-lg font-semibold tracking-tight">
            vymalo dashboard
          </Link>
          <nav className="hidden gap-1 md:flex">
            {links.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className={cn(
                  "rounded-md px-3 py-2 text-sm font-medium text-muted-foreground transition hover:bg-muted hover:text-foreground",
                  pathname === link.href && "bg-muted text-foreground shadow-sm"
                )}
              >
                {link.label}
              </Link>
            ))}
          </nav>
        </div>
        <div className="flex items-center gap-3">
          {loading ? (
            <span
              className="h-4 w-4 animate-spin rounded-full border-2 border-muted-foreground/40 border-t-foreground"
              aria-label="Loading session"
            />
          ) : session ? (
            <div className="flex items-center gap-2 rounded-full border bg-card px-3 py-1.5 shadow-sm">
              <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-sm font-semibold text-primary-foreground">
                {session.user.name?.[0] ?? "U"}
              </div>
              <div className="text-left">
                <div className="text-xs text-muted-foreground">Signed in</div>
                <div className="text-sm font-semibold leading-tight">{session.user.email}</div>
              </div>
            </div>
          ) : (
            <Button asChild size="sm" variant="outline">
              <Link href="/login">Sign in</Link>
            </Button>
          )}
        </div>
      </div>
    </header>
  );
}
