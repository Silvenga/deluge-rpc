use clap::Subcommand;
use deluge_rpc::{DelugeClient, LabelRpc};

#[derive(Subcommand, Debug, Clone)]
pub enum LabelCommand {
    List,
    Add { id: String },
    Remove { id: String },
}

pub async fn run_label(client: &DelugeClient, cmd: &LabelCommand) -> anyhow::Result<String> {
    match cmd {
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

#[derive(Subcommand, Debug, Clone)]
pub enum PluginsCommand {
    #[command(flatten)]
    Label(LabelCommand),
}
