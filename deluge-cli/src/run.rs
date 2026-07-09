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
            Command::Call(cmd) => match cmd.run(&client).await {
                Ok(response) => {
                    let plain = rencode_to_plain_json(&response);
                    let output =
                        serde_json::to_string_pretty(&plain).unwrap_or_else(|_| "null".to_owned());
                    info!("{output}");
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Command::Daemon(cmd) => cmd.run(&client).await.map(|output| info!("{output}")),
            Command::Core(cmd) => cmd.run(&client).await.map(|output| info!("{output}")),
            Command::Plugin(cmd) => cmd.run(&client).await.map(|output| info!("{output}")),
            Command::Status(cmd) => cmd.run(&client).await.map(|output| info!("{output}")),
        }
    };

    if let Some(recorder) = recorder {
        if let Err(e) = recorder.persist().await {
            tracing::warn!(error = ?e, "failed to persist cassette");
        }
    }

    result
}
