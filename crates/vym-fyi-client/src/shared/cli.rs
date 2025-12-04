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

    /// List all short links visible to this client.
    LinksList,
}
