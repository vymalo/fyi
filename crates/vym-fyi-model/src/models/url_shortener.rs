use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Identifier for a tenant (logical client).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

/// Identifier for a short link.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShortLinkId(pub String);

/// Identifier for an API key (database-level id, not the secret).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiKeyId(pub String);

/// Role associated with an API key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Url,
}

/// Tenant represents an isolated client of the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub status: TenantStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantStatus {
    Active,
    Suspended,
}

/// ApiKey represents a hashed API key bound to a tenant and role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: ApiKeyId,
    pub tenant_id: TenantId,
    /// Human-readable name for the key (e.g. "cli-prod").
    pub name: String,
    /// Hash of the API key secret.
    pub key_hash: String,
    pub role: Role,
}

/// ShortLink maps a slug to a target URL within a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortLink {
    pub id: ShortLinkId,
    pub tenant_id: TenantId,
    pub slug: String,
    pub target_url: String,
    pub is_active: bool,
}

/// CLI configuration file (config.yaml) structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server: ServerSection,
    pub clients: HashMap<String, ClientEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSection {
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEntry {
    /// Display name for this client configuration.
    pub name: String,
    /// API key value, possibly containing env placeholders like "$(CLIENT_A_SECRET)".
    pub api_key: String,
    /// Optional role information (informational for the CLI).
    pub role: Option<Role>,
}
