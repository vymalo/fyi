use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use crate::models::errors::{AppError, AppResult};
use crate::models::url_shortener::{ClientConfig, ClientEntry};

/// Load a `ClientConfig` (config.yaml) from the given path.
pub fn load_client_config(path: &Path) -> AppResult<ClientConfig> {
    let contents = fs::read_to_string(path)?;
    let config: ClientConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}

/// Resolve `$(ENV_VAR_NAME)` placeholders inside a string.
pub fn resolve_env_placeholders(input: &str) -> AppResult<String> {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'$' && index + 1 < bytes.len() && bytes[index + 1] == b'(' {
            // We found a potential $(NAME) placeholder.
            let start = index + 2;
            let mut end = start;
            while end < bytes.len() && bytes[end] != b')' {
                end += 1;
            }

            if end >= bytes.len() {
                // No closing ')', treat the rest as literal.
                result.push_str(&input[index..]);
                break;
            }

            let var_name = &input[start..end];
            let env_value = std::env::var(var_name).map_err(|_| AppError::MissingEnvVar {
                name: var_name.to_string(),
            })?;
            result.push_str(&env_value);

            index = end + 1;
        } else {
            result.push(bytes[index] as char);
            index += 1;
        }
    }

    Ok(result)
}

/// Representation of a resolved client configuration.
#[derive(Debug, Clone)]
pub struct ResolvedClient {
    pub id: String,
    pub base_url: String,
    pub entry: ClientEntry,
    /// Optional resolved master API key (if configured on the server section).
    pub master_api_key: Option<String>,
}

/// Resolve a client entry by id from a loaded config, applying env substitution.
pub fn resolve_client(config: &ClientConfig, client_id: &str) -> AppResult<ResolvedClient> {
    let entry = config
        .clients
        .get(client_id)
        .cloned()
        .ok_or_else(|| AppError::Config(format!("Unknown client id: {}", client_id)))?;

    let resolved_api_key = resolve_env_placeholders(&entry.api_key)?;

    let mut resolved_entry = entry.clone();
    resolved_entry.api_key = resolved_api_key;

    let master_api_key = match &config.server.master_api_key {
        Some(raw) => Some(resolve_env_placeholders(raw)?),
        None => None,
    };

    Ok(ResolvedClient {
        id: client_id.to_string(),
        base_url: config.server.base_url.clone(),
        entry: resolved_entry,
        master_api_key,
    })
}

pub fn bind_addr_from_env(default_port: u16) -> AppResult<SocketAddr> {
    let host = std::env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(default_port);

    let addr = format!("{}:{}", host, port);
    addr.parse()
        .map_err(|e| AppError::Config(format!("invalid bind address {}: {}", addr, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_env_placeholders() {
        unsafe { std::env::set_var("TEST_KEY", "secret-value") };
        let raw = "value-$(TEST_KEY)-tail";
        let resolved = resolve_env_placeholders(raw).expect("placeholder should resolve");
        assert_eq!(resolved, "value-secret-value-tail");
    }

    #[test]
    fn missing_env_placeholder_errors() {
        unsafe { std::env::remove_var("MISSING_KEY") };
        let err = resolve_env_placeholders("$(MISSING_KEY)").unwrap_err();
        match err {
            AppError::MissingEnvVar { name } => assert_eq!(name, "MISSING_KEY"),
            other => panic!("unexpected error: {other}"),
        }
    }
}
