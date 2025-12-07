# Model crate helpers

This crate bundles shared code (models, services, helpers) that the server, webhook, and client reuse.

Logging helper

- `services/logging.rs` exposes `setup_logging(service_name: &str)` which installs a basic `tracing_subscriber::fmt()` subscriber.
- `RUST_LOG` controls filtering (defaults to `info` if unset). Example: `info,hyper=warn,reqwest=warn`.
- The `service_name` parameter is currently informational; pass the component name for clarity or future tagging.

Usage

```rust
use vym_fyi_model::services::logging::setup_logging;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging("my-service")?;
    // …
    Ok(())
}
```

Other shared services include:

- `services/http_client.rs`: tuned `reqwest` client helper with pooling/timeouts.
