use std::fs;
use std::path::Path;

use vym_fyi_model::models::errors::{AppError, AppResult};
use vym_fyi_model::models::url_shortener::{ClientConfig, ClientEntry};

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

pub struct ResolvedClient {
    pub id: String,
    pub base_url: String,
    pub entry: ClientEntry,
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

    Ok(ResolvedClient {
        id: client_id.to_string(),
        base_url: config.server.base_url.clone(),
        entry: resolved_entry,
    })
}
