use crate::config::{Cli, Command};
use crate::helpers::rencode_to_plain_json;
use crate::recording::Recorder;
use clap::Parser;
use deluge_rpc_client::DelugeClientBuilder;
use tracing::info;

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut builder = DelugeClientBuilder::new(
        cli.config.host,
        cli.config.port,
        cli.config.username,
        cli.config.password,
    );

    let recorder = if let Some(record_path) = &cli.config.record {
        let mut recorder = Recorder::new(record_path);
        builder = recorder.configure_client(builder);
        Some(recorder)
    } else {
        None
    };

    let result: anyhow::Result<()> = {
        let client = builder.build();
        match &cli.command {
            Command::Call(cmd) => {
                let response = cmd.run(&client).await?;
                let plain = rencode_to_plain_json(&response);
                let output =
                    serde_json::to_string_pretty(&plain).unwrap_or_else(|_| "null".to_owned());
                info!("{output}");
            }
            Command::Daemon(cmd) => {
                let output = cmd.run(&client).await?;
                info!("{output}");
            }
            Command::Core(cmd) => {
                let output = cmd.run(&client).await?;
                info!("{output}");
            }
            Command::Plugin(cmd) => {
                let output = cmd.run(&client).await?;
                info!("{output}");
            }
        }
        Ok(())
    };

    if let Some(recorder) = recorder {
        recorder.persist().await?;
    }

    result
}
