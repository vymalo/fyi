import { useMemo, type PropsWithChildren, createContext, useContext } from "react";
import { createAuthClient, type Session } from "better-auth/client";

export type AuthClientConfig = {
  baseUrl: string;
  apiKey?: string;
  secret?: string;
};

export type AuthClient = ReturnType<typeof createAuthClient>;
export type AuthSession = Session;

export type AuthContextValue = {
  client: AuthClient;
  session: Session | null;
  loading: boolean;
};

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export function buildAuthHeaders(config: AuthClientConfig): Record<string, string> {
  const headers: Record<string, string> = {};

  if (config.apiKey) {
    headers.Authorization = `Bearer ${config.apiKey}`;
  }

  if (config.secret) {
    headers["x-better-auth-secret"] = config.secret;
  }

  return headers;
}

export function createBetterAuthClient(config: AuthClientConfig): AuthClient {
  return createAuthClient({
    baseURL: config.baseUrl,
    fetchOptions: {
      credentials: "include",
      headers: buildAuthHeaders(config),
    },
  });
}

export function AuthProvider({ config, children }: PropsWithChildren<{ config: AuthClientConfig }>) {
  const client = useMemo(() => createBetterAuthClient(config), [config]);
  const { data, isPending } = client.useSession();
  const session = (data as Session | null) ?? null;

  return (
    <AuthContext.Provider value={{ client, session, loading: isPending }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);

  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }

  return context;
}
