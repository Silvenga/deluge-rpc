use clap::Subcommand;
use deluge_rpc_client::{DelugeClient, LabelRpc};

/// Plugin RPC methods. Plugin methods only exist when the plugin is enabled.
/// Check via `core.get_enabled_plugins()` or use `daemon.get_method_list()`.
#[derive(Subcommand, Debug, Clone)]
pub enum PluginsCommand {
    #[command(flatten)]
    Label(LabelCommand),
}

impl PluginsCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            PluginsCommand::Label(cmd) => cmd.run(client).await,
        }
    }
}

/// `label.*` plugin methods - assign labels to torrents and manage labels.
#[derive(Subcommand, Debug, Clone)]
pub enum LabelCommand {
    /// List all label IDs (sorted).
    List,
    /// Create a new label with default options.
    Add {
        /// Label ID to create.
        id: String,
    },
    /// Remove a label. Torrents with that label lose it.
    Remove {
        /// Label ID to remove.
        id: String,
    },
}

impl LabelCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            LabelCommand::List => {
                let labels = client.plugins().label.get_labels().await?;
                Ok(serde_json::to_string_pretty(&labels)?)
            }
            LabelCommand::Add { id } => {
                client.plugins().label.add(id).await?;
                Ok(format!("Label '{id}' added."))
            }
            LabelCommand::Remove { id } => {
                client.plugins().label.remove(id).await?;
                Ok(format!("Label '{id}' removed."))
            }
        }
    }
}
