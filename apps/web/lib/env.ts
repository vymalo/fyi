export function getApiBaseUrl(): string {
  return process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3001";
}

export function getAuthSecret(): string | undefined {
  return process.env.BETTER_AUTH_SECRET;
}

export function getAuthApiKey(): string | undefined {
  return process.env.NEXT_PUBLIC_BETTER_AUTH_API_KEY;
}
