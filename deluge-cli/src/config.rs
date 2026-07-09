use crate::commands::{CallCommand, CoreCommand, DaemonCommand, PluginsCommand, StatusCommand};
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "deluge-cli",
    version,
    about = "CLI client for the Deluge daemon RPC protocol",
    author = "Mark Lopez <m@silvenga.com>"
)]
pub struct Cli {
    #[command(flatten)]
    pub config: HostConfig,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Call a raw RPC method directly by name with JSON args/kwargs.
    Call(CallCommand),
    /// `daemon.*` methods (e.g., daemon info, version, methods, shutdown).
    #[command(subcommand)]
    Daemon(DaemonCommand),
    /// `core.*` methods (e.g., torrents, session, config, plugins).
    #[command(subcommand)]
    Core(CoreCommand),
    /// Plugin RPC methods (e.g., label, etc.). Plugin methods require the plugin to be enabled.
    #[command(subcommand)]
    Plugin(PluginsCommand),
    /// High-level status overview.
    Status(StatusCommand),
}

#[derive(Args, Debug, Clone)]
pub struct HostConfig {
    /// The deluge daemon host.
    #[arg(long, short = 'H', env = "DELUGE_HOST", default_value = "127.0.0.1")]
    pub host: String,
    /// The deluge daemon port.
    #[arg(short = 'P', long, env = "DELUGE_PORT", default_value_t = 58846)]
    pub port: u16,
    /// The deluge daemon username.
    #[arg(short, long, env = "DELUGE_USERNAME", default_value = "localclient")]
    pub username: String,
    /// The deluge daemon password.
    #[arg(short, long, env = "DELUGE_PASSWORD")]
    pub password: String,
    /// File path to record RPC calls to.
    #[arg(long)]
    pub record: Option<String>,
}
