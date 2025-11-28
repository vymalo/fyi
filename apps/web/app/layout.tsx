import "./globals.css";

import type { Metadata } from "next";
import { Inter } from "next/font/google";
import { Navbar } from "../components/navbar";
import { Providers } from "../providers";
import { getAuthConfig } from "../lib/auth-client";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Vymalo Dashboard",
  description: "Workspace-aware Next.js starter powered by Tailwind, shadcn/ui, and daisyUI.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  const authConfig = getAuthConfig();

  return (
    <html lang="en" data-theme="corporate">
      <body className={inter.className}>
        <Providers config={authConfig}>
          <Navbar />
          <main className="container mx-auto px-4 pb-10 pt-6">{children}</main>
        </Providers>
      </body>
    </html>
  );
}
