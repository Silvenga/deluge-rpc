use crate::cli_config::CliConfig;
use crate::commands::{CallCommand, CoreCommand, DaemonCommand, PluginsCommand};
use crate::helpers::rencode_to_plain_json;
use crate::record::{
    Cassette, Interaction, Request, Response, load_cassette, write_cassette_atomic,
};
use anyhow::Context;
use clap::{Parser, Subcommand};
use deluge_rpc_client::{DelugeClientBuilder, RecordedInteraction, RecordedResponse, RencodeValue};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(
    name = "deluge-cli",
    version,
    about = "CLI client for the Deluge daemon RPC protocol"
)]
struct Cli {
    #[command(flatten)]
    config: CliConfig,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    Call(CallCommand),
    #[command(subcommand)]
    Daemon(DaemonCommand),
    #[command(subcommand)]
    Core(CoreCommand),
    #[command(subcommand)]
    Plugin(PluginsCommand),
}

#[expect(clippy::print_stdout, reason = "CLI prints command output to stdout")]
pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let resolved = cli.config.resolve()?;

    let (recorder_tx, mut recorder_rx) = mpsc::channel::<RecordedInteraction>(256);

    let mut builder =
        DelugeClientBuilder::new(resolved.host, resolved.port, resolved.user, resolved.pass);

    if resolved.record.is_some() {
        builder = builder.with_recorder(recorder_tx);
    }

    let client = builder.build();

    let command_result: anyhow::Result<()> = async {
        match &cli.command {
            Command::Call(cmd) => {
                let response = cmd.run(&client).await?;
                let plain = rencode_to_plain_json(&response);
                let output =
                    serde_json::to_string_pretty(&plain).unwrap_or_else(|_| "null".to_owned());
                println!("{output}");
            }
            Command::Daemon(cmd) => {
                let output = cmd.run(&client).await?;
                println!("{output}");
            }
            Command::Core(cmd) => {
                let output = cmd.run(&client).await?;
                println!("{output}");
            }
            Command::Plugin(cmd) => {
                let output = cmd.run(&client).await?;
                println!("{output}");
            }
        }
        Ok(())
    }
    .await;

    drop(client);

    if let Some(record_path) = &resolved.record {
        let mut interactions = Vec::new();
        while let Some(recorded) = recorder_rx.recv().await {
            interactions.push(to_interaction(recorded));
        }

        let path = PathBuf::from(record_path);

        let mut existing = Vec::new();
        if path.exists() {
            let cassette = load_cassette(&path).context("failed to load existing cassette")?;
            existing = cassette
                .interactions
                .into_iter()
                .filter(|i| i.request.method != "daemon.login")
                .collect::<Vec<_>>();
        }

        interactions.retain(|i| i.request.method != "daemon.login");
        existing.extend(interactions);

        let cassette = Cassette {
            version: 1,
            recorded_at: chrono::Utc::now().to_rfc3339(),
            daemon_version: None,
            interactions: existing,
        };
        write_cassette_atomic(&path, &cassette).context("failed to write cassette file")?;
        tracing::info!("cassette written to {}", record_path);
    }

    command_result
}

fn to_interaction(recorded: RecordedInteraction) -> Interaction {
    Interaction {
        request: Request {
            method: recorded.request.method,
            args: RencodeValue::List(recorded.request.args),
            kwargs: RencodeValue::Dict(recorded.request.kwargs),
        },
        response: match recorded.response {
            RecordedResponse::Ok { value } => Response::Ok { value },
            RecordedResponse::Error {
                exc_type,
                exc_msg,
                traceback,
            } => Response::Error {
                exc_type,
                exc_msg,
                traceback,
            },
        },
    }
}
