use clap::Subcommand;
use deluge_rpc_client::DelugeClient;

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
    /// Subscribe the session to event names (full-replace operation).
    Events {
        /// JSON array of event names to subscribe to (e.g. `["TorrentAddedEvent"]`).
        names: String,
    },
    /// Check whether the current session can call a named RPC method.
    Authorized {
        /// The RPC method name to check (e.g. `core.get_config`).
        rpc: String,
    },
}

impl DaemonCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            DaemonCommand::Info => {
                let info = client.daemon.info().await?;
                Ok(serde_json::to_string_pretty(&info)?)
            }
            DaemonCommand::Version => {
                let version = client.daemon.get_version().await?;
                Ok(serde_json::to_string_pretty(&version)?)
            }
            DaemonCommand::Methods => {
                let methods = client.daemon.get_method_list().await?;
                Ok(serde_json::to_string_pretty(&methods)?)
            }
            DaemonCommand::Shutdown => {
                client.daemon.shutdown().await?;
                Ok("Shutdown requested.".to_owned())
            }
            DaemonCommand::Events { names } => {
                let event_names: Vec<String> = serde_json::from_str(names)
                    .map_err(|e| anyhow::anyhow!("failed to parse event names JSON: {e}"))?;
                let result = client.daemon.set_event_interest(&event_names).await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            DaemonCommand::Authorized { rpc } => {
                let result = client.daemon.authorized_call(rpc).await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
        }
    }
}
