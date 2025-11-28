"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
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
    <div className="navbar sticky top-0 z-20 bg-base-100/80 backdrop-blur">
      <div className="flex-1">
        <Link href="/" className="btn btn-ghost text-xl font-black">
          vymalo dashboard
        </Link>
        <div className="hidden gap-2 md:flex">
          {links.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className={`btn btn-ghost btn-sm font-semibold ${
                pathname === link.href ? "bg-base-200" : ""
              }`}
            >
              {link.label}
            </Link>
          ))}
        </div>
      </div>
      <div className="flex items-center gap-2">
        {loading ? (
          <span className="loading loading-dots loading-sm" aria-label="Loading session" />
        ) : session ? (
          <div className="flex items-center gap-2 rounded-lg border border-base-200 px-3 py-2 text-sm">
            <div className="avatar placeholder h-8 w-8">
              <div className="rounded-full bg-primary text-primary-content">{session.user.name?.[0] ?? "U"}</div>
            </div>
            <div className="text-left">
              <div className="text-xs text-base-content/70">Signed in</div>
              <div className="font-semibold leading-tight">{session.user.email}</div>
            </div>
          </div>
        ) : (
          <Link href="/login" className="btn btn-outline btn-sm font-semibold">
            Sign in
          </Link>
        )}
      </div>
    </div>
  );
}
