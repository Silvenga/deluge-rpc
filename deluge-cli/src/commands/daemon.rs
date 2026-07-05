use clap::Subcommand;
use deluge_rpc::{DaemonRpc, DelugeClient};

#[derive(Subcommand, Debug, Clone)]
pub enum DaemonCommand {
    Info,
    Version,
    Methods,
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
