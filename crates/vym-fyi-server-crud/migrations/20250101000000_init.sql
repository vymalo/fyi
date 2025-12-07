-- Base schema for vymalo-fyi URL shortener.

CREATE TABLE IF NOT EXISTS tenants (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    status text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS api_keys (
    id uuid PRIMARY KEY,
    tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name text NOT NULL,
    key_hash text NOT NULL,
    role text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    revoked_at timestamptz
);

-- Short links are resolved by slug on the redirect server.
-- Slug is globally unique; tenant_id still associates the record
-- with a tenant for management purposes.
CREATE TABLE IF NOT EXISTS short_links (
    slug text PRIMARY KEY,
    tenant_id uuid REFERENCES tenants(id),
    target_url text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    created_by_api_key_id uuid REFERENCES api_keys(id),
    expires_at timestamptz,
    is_active boolean NOT NULL DEFAULT true
);
