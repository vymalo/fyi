"use client";

import { useState } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import type { AuthClientConfig } from "@vymalo/auth";
import { AuthProvider } from "@vymalo/auth";
import { createQueryClient } from "./lib/query-client";

export function Providers({ config, children }: React.PropsWithChildren<{ config: AuthClientConfig }>) {
  const [queryClient] = useState(() => createQueryClient());

  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider config={config}>{children}</AuthProvider>
      <ReactQueryDevtools initialIsOpen={false} buttonPosition="bottom-left" />
    </QueryClientProvider>
  );
}
