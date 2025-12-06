use std::fs;

use tempfile::tempdir;
use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::services::config::{load_client_config, resolve_client};

#[test]
fn loads_and_resolves_client_config() -> AppResult<()> {
    let dir = tempdir()?;
    let path = dir.path().join("config.yaml");

    let yaml = r#"
server:
  base_url: "https://example.test"
  master_api_key: "$(MASTER_KEY)"

clients:
  client-a:
    name: "Client A"
    api_key: "$(CLIENT_A)"
"#;

    fs::write(&path, yaml)?;

    unsafe {
        std::env::set_var("MASTER_KEY", "master-123");
        std::env::set_var("CLIENT_A", "client-a-key");
    }

    let config = load_client_config(&path)?;
    let resolved = resolve_client(&config, "client-a")?;

    assert_eq!(resolved.base_url, "https://example.test");
    assert_eq!(resolved.entry.api_key, "client-a-key");
    assert_eq!(resolved.master_api_key.as_deref(), Some("master-123"));
    assert_eq!(resolved.id, "client-a");

    Ok(())
}
