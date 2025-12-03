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
    #[arg(long, short, env = "VYM_FYI_CLIENT")]
    pub client: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Check connectivity to the CRUD server using the selected client.
    Ping,
    // Future subcommands: links, tenants, api-keys, etc.
}
