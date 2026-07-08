use clap::Subcommand;
use deluge_rpc_client::{DaemonRpc, DelugeClient};

/// `daemon.*` methods (e.g., daemon info, version, and shutdown).
#[derive(Subcommand, Debug, Clone)]
pub enum DaemonCommand {
    /// Get the daemon version string (pre-auth handshake value, same as `daemon.get_version`).
    Info,
    /// Get the daemon version string (e.g. `"2.1.1"`).
    Version,
    /// List all registered RPC method names (`"<object>.<method>"` format).
    /// Only includes `@export`-registered methods - excludes the hardcoded
    /// `daemon.info`, `daemon.login`, and `daemon.set_event_interest`.
    Methods,
    /// Shut down the daemon. The response is unreliable.
    Shutdown,
}

impl DaemonCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            DaemonCommand::Info => {
                let info = client.daemon().info().await?;
                Ok(serde_json::to_string_pretty(&info)?)
            }
            DaemonCommand::Version => {
                let version = client.daemon().get_version().await?;
                Ok(serde_json::to_string_pretty(&version)?)
            }
            DaemonCommand::Methods => {
                let methods = client.daemon().get_method_list().await?;
                Ok(serde_json::to_string_pretty(&methods)?)
            }
            DaemonCommand::Shutdown => {
                client.daemon().shutdown().await?;
                Ok("Shutdown requested.".to_owned())
            }
        }
    }
}
