use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    version,
    name = "vym-fyi-client",
    about = "CLI client for the vymalo URL shortener"
)]
pub struct Opt {
    /// Path to config.yaml (can also be set via VYM_FYI_CONFIG).
    #[arg(long, short, env = "VYM_FYI_CONFIG", default_value = "config.yaml")]
    pub config: PathBuf,

    /// Client id to use from the `clients` map (can also be set via VYM_FYI_CLIENT).
    /// If omitted and --use-master is set, only the master_api_key is used.
    #[arg(long, short = 'i', env = "VYM_FYI_CLIENT")]
    pub client: String,

    /// Use the master API key instead of the client's API key when available.
    #[arg(long)]
    pub use_master: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Check connectivity to the CRUD server using the selected client.
    Ping,

    /// Create a new short link (slug -> target URL).
    LinksCreate {
        /// Optional slug part of the short URL (e.g. 'promo-2025').
        /// If omitted, the server will generate a random slug.
        #[arg(long)]
        slug: Option<String>,

        /// Target URL that the slug should redirect to.
        #[arg(long)]
        target: String,
    },

    /// List short links visible to this client (with optional filters).
    LinksList {
        /// Page number (1-based).
        #[arg(long)]
        page: Option<u32>,

        /// Items per page (default 20, max 100).
        #[arg(long = "per-page")]
        per_page: Option<u32>,

        /// Filter by exact slug.
        #[arg(long)]
        slug: Option<String>,

        /// Filter by target URL containing this substring (case-insensitive).
        #[arg(long = "target-contains")]
        target_contains: Option<String>,

        /// Filter by active status.
        #[arg(long)]
        active: Option<bool>,

        /// Only include links created before this timestamp (RFC3339, e.g. 2025-01-01T00:00:00Z).
        #[arg(long = "created-before")]
        created_before: Option<String>,

        /// Only include links created after this timestamp (RFC3339).
        #[arg(long = "created-after")]
        created_after: Option<String>,

        /// Only include links expiring before this timestamp (RFC3339).
        #[arg(long = "expires-before")]
        expires_before: Option<String>,

        /// Only include links expiring after this timestamp (RFC3339).
        #[arg(long = "expires-after")]
        expires_after: Option<String>,
    },
}
