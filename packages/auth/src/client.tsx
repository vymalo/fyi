import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type PropsWithChildren,
} from "react";

export type AuthUser = {
  id: string;
  email: string;
  name?: string;
};

export type AuthSession = {
  user: AuthUser;
  token?: string;
};

export type AuthClientConfig = {
  baseUrl: string;
  apiKey?: string;
  secret?: string;
};

export type AuthClient = {
  getSession: () => Promise<AuthSession | null>;
  signIn: (email: string) => Promise<AuthSession>;
  signOut: () => Promise<void>;
};

function buildSessionEndpoint(baseUrl: string): string {
  return `${baseUrl.replace(/\/$/, "")}/auth/session`;
}

export function createAuthClient(config: AuthClientConfig): AuthClient {
  const sessionEndpoint = buildSessionEndpoint(config.baseUrl);
  const headers: Record<string, string> = {};

  if (config.apiKey) {
    headers.Authorization = `Bearer ${config.apiKey}`;
  }

  if (config.secret) {
    headers["x-better-auth-secret"] = config.secret;
  }

  return {
    async getSession() {
      const response = await fetch(sessionEndpoint, { headers }).catch(() => null);

      if (!response?.ok) {
        return null;
      }

      const data = (await response.json()) as AuthSession | null;
      return data ?? null;
    },
    async signIn(email) {
      const now = Date.now().toString(36);
      return {
        user: { id: now, email, name: email.split("@")[0] ?? email },
        token: headers.Authorization ?? `demo-${now}`,
      };
    },
    async signOut() {
      await Promise.resolve();
    },
  };
}

export type AuthContextValue = {
  client: AuthClient;
  session: AuthSession | null;
  loading: boolean;
};

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export function AuthProvider({ config, children }: PropsWithChildren<{ config: AuthClientConfig }>) {
  const client = useMemo(() => createAuthClient(config), [config]);
  const [session, setSession] = useState<AuthSession | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let active = true;

    client
      .getSession()
      .then((value) => {
        if (active) {
          setSession(value);
        }
      })
      .finally(() => {
        if (active) {
          setLoading(false);
        }
      });

    return () => {
      active = false;
    };
  }, [client]);

  return <AuthContext.Provider value={{ client, session, loading }}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);

  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }

  return context;
}

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
