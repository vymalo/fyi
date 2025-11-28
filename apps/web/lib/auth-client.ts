import type { AuthClientConfig } from "@vymalo/auth";
import { createBetterAuthClient } from "@vymalo/auth";
import { getApiBaseUrl, getAuthApiKey, getAuthSecret } from "./env";

export function getAuthConfig(): AuthClientConfig {
  return {
    baseUrl: getApiBaseUrl(),
    apiKey: getAuthApiKey(),
    secret: getAuthSecret(),
  };
}

export function buildAuthClient() {
  return createBetterAuthClient(getAuthConfig());
}
