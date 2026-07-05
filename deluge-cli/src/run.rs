use crate::cli_config::CliConfig;
use crate::commands::{
    CallCommand, CoreCommand, CoreConfigCommand, CoreSessionCommand, CoreTorrentsCommand,
    DaemonCommand, LabelCommand, PluginsCommand, PluginsListCommand,
};
use crate::helpers::{rencode_from_json_value, rencode_to_plain_json};
use crate::record::{
    load_cassette, write_cassette_atomic, Cassette, Interaction, Request, Response,
};
use anyhow::Context;
use clap::{Parser, Subcommand};
use deluge_rpc::{DelugeClient, DelugeRpcRequest, RencodeValue};
use std::collections::BTreeMap;
use std::path::PathBuf;

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

    let client = DelugeClient::connect(
        &resolved.host,
        resolved.port,
        &resolved.user,
        &resolved.pass,
    )
    .await
    .context("failed to connect to Deluge daemon")?;

    let mut interactions: Vec<Interaction> = Vec::new();

    match &cli.command {
        Command::Call(cmd) => {
            let (method, parsed_args, parsed_kwargs, response) = cmd.run(&client).await?;
            let plain = rencode_to_plain_json(&response);
            let output = serde_json::to_string_pretty(&plain).unwrap_or_else(|_| "null".to_owned());
            println!("{output}");

            if resolved.record.is_some() {
                interactions.push(Interaction {
                    request: Request {
                        method,
                        args: RencodeValue::List(parsed_args),
                        kwargs: RencodeValue::Dict(parsed_kwargs),
                    },
                    response: Response::Ok { value: response },
                });
            }
        }
        Command::Daemon(cmd) => {
            let output = cmd.run(&client).await?;
            println!("{output}");

            if resolved.record.is_some() {
                let (method, rpc_args, rpc_kwargs, raw_response) =
                    daemon_record_call(&client, cmd).await?;
                interactions.push(Interaction {
                    request: Request {
                        method,
                        args: RencodeValue::List(rpc_args),
                        kwargs: RencodeValue::Dict(rpc_kwargs),
                    },
                    response: Response::Ok {
                        value: raw_response,
                    },
                });
            }
        }
        Command::Core(cmd) => {
            let output = cmd.run(&client).await?;
            println!("{output}");

            if resolved.record.is_some() {
                let interactions_from_core = core_record_call(&client, cmd).await?;
                interactions.extend(interactions_from_core);
            }
        }
        Command::Plugin(cmd) => {
            let output = cmd.run(&client).await?;
            println!("{output}");
        }
    }

    if let Some(record_path) = &resolved.record {
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

    Ok(())
}

async fn daemon_record_call(
    client: &DelugeClient,
    cmd: &DaemonCommand,
) -> anyhow::Result<(
    String,
    Vec<RencodeValue>,
    BTreeMap<RencodeValue, RencodeValue>,
    RencodeValue,
)> {
    use BTreeMap;
    use RencodeValue;

    let (method, rpc_args, rpc_kwargs): (
        &str,
        Vec<RencodeValue>,
        BTreeMap<RencodeValue, RencodeValue>,
    ) = match cmd {
        DaemonCommand::Info => ("daemon.info", vec![], BTreeMap::new()),
        DaemonCommand::Version => ("daemon.get_version", vec![], BTreeMap::new()),
        DaemonCommand::Methods => ("daemon.get_method_list", vec![], BTreeMap::new()),
        DaemonCommand::Shutdown => ("daemon.shutdown", vec![], BTreeMap::new()),
    };

    let request = DelugeRpcRequest::new(method)
        .with_args(rpc_args.clone())
        .with_kwargs(rpc_kwargs.clone());
    let response = client.call(request).await?;

    Ok((method.to_owned(), rpc_args, rpc_kwargs, response))
}

async fn core_record_call(
    client: &DelugeClient,
    cmd: &CoreCommand,
) -> anyhow::Result<Vec<Interaction>> {
    use BTreeMap;
    use RencodeValue;

    let mut interactions = Vec::new();
    let args_vec: Vec<RencodeValue>;
    let kwargs: BTreeMap<RencodeValue, RencodeValue>;

    let method = match cmd {
        CoreCommand::FreeSpace { path } => {
            args_vec = match path {
                Some(p) => vec![RencodeValue::Str(p.clone())],
                None => vec![RencodeValue::None],
            };
            kwargs = BTreeMap::new();
            "core.get_free_space"
        }
        CoreCommand::Torrents(sub) => {
            let (m, a, kw) = core_torrents_record_params(sub)?;
            args_vec = a;
            kwargs = kw;
            m
        }
        CoreCommand::Session(CoreSessionCommand::Status { keys }) => {
            let keys_list: Vec<RencodeValue> = parse_record_keys(keys)
                .into_iter()
                .map(RencodeValue::Str)
                .collect();
            args_vec = vec![RencodeValue::List(keys_list)];
            kwargs = BTreeMap::new();
            "core.get_session_status"
        }
        CoreCommand::Config(sub) => {
            let (m, a, kw) = core_config_record_params(sub)?;
            args_vec = a;
            kwargs = kw;
            m
        }
        CoreCommand::Plugins(sub) => {
            let (m, a, kw) = core_plugins_record_params(sub);
            args_vec = a;
            kwargs = kw;
            m
        }
    };

    let request = DelugeRpcRequest::new(method)
        .with_args(args_vec.clone())
        .with_kwargs(kwargs.clone());
    let response = client.call(request).await?;

    interactions.push(Interaction {
        request: Request {
            method: method.to_owned(),
            args: RencodeValue::List(args_vec),
            kwargs: RencodeValue::Dict(kwargs),
        },
        response: Response::Ok { value: response },
    });

    Ok(interactions)
}

fn core_torrents_record_params(
    cmd: &CoreTorrentsCommand,
) -> anyhow::Result<(
    &'static str,
    Vec<RencodeValue>,
    BTreeMap<RencodeValue, RencodeValue>,
)> {
    use BTreeMap;
    use RencodeValue;

    match cmd {
        CoreTorrentsCommand::List { filter, keys } => {
            let filter_value = match filter {
                Some(f) => {
                    let json: serde_json::Value = serde_json::from_str(f)
                        .map_err(|e| anyhow::anyhow!("failed to parse filter JSON: {e}"))?;
                    deluge_rpc::from_json(&json)
                        .map_err(|e| anyhow::anyhow!("failed to convert filter: {e}"))?
                }
                None => RencodeValue::Dict(BTreeMap::new()),
            };
            let keys_list: Vec<RencodeValue> = parse_record_keys(keys)
                .into_iter()
                .map(RencodeValue::Str)
                .collect();
            let mut kwargs = BTreeMap::new();
            kwargs.insert(RencodeValue::Str("diff".into()), RencodeValue::Bool(false));
            Ok((
                "core.get_torrents_status",
                vec![filter_value, RencodeValue::List(keys_list)],
                kwargs,
            ))
        }
        CoreTorrentsCommand::Status { torrent_id, keys } => {
            let keys_list: Vec<RencodeValue> = parse_record_keys(keys)
                .into_iter()
                .map(RencodeValue::Str)
                .collect();
            let mut kwargs = BTreeMap::new();
            kwargs.insert(RencodeValue::Str("diff".into()), RencodeValue::Bool(false));
            Ok((
                "core.get_torrent_status",
                vec![
                    RencodeValue::Str(torrent_id.clone()),
                    RencodeValue::List(keys_list),
                ],
                kwargs,
            ))
        }
        CoreTorrentsCommand::Remove {
            torrent_id,
            keep_data,
        } => Ok((
            "core.remove_torrent",
            vec![
                RencodeValue::Str(torrent_id.clone()),
                RencodeValue::Bool(!keep_data),
            ],
            BTreeMap::new(),
        )),
    }
}

fn core_config_record_params(
    cmd: &CoreConfigCommand,
) -> anyhow::Result<(
    &'static str,
    Vec<RencodeValue>,
    BTreeMap<RencodeValue, RencodeValue>,
)> {
    use BTreeMap;
    use RencodeValue;

    match cmd {
        CoreConfigCommand::Get { key: Some(k) } => Ok((
            "core.get_config_value",
            vec![RencodeValue::Str(k.clone())],
            BTreeMap::new(),
        )),
        CoreConfigCommand::Get { key: None } => Ok(("core.get_config", vec![], BTreeMap::new())),
        CoreConfigCommand::Set { json } => {
            let parsed: serde_json::Value = serde_json::from_str(json)
                .map_err(|e| anyhow::anyhow!("failed to parse config JSON: {e}"))?;
            let obj = parsed
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("config set value must be a JSON object"))?;
            let mut config = BTreeMap::new();
            for (k, v) in obj {
                let rencode_val = rencode_from_json_value(v)?;
                config.insert(RencodeValue::Str(k.clone()), rencode_val);
            }
            Ok((
                "core.set_config",
                vec![RencodeValue::Dict(config)],
                BTreeMap::new(),
            ))
        }
    }
}

fn core_plugins_record_params(
    cmd: &PluginsListCommand,
) -> (
    &'static str,
    Vec<RencodeValue>,
    BTreeMap<RencodeValue, RencodeValue>,
) {
    use BTreeMap;
    use RencodeValue;

    match cmd {
        PluginsListCommand::List => ("core.get_enabled_plugins", vec![], BTreeMap::new()),
        PluginsListCommand::Enable { name } => (
            "core.enable_plugin",
            vec![RencodeValue::Str(name.clone())],
            BTreeMap::new(),
        ),
        PluginsListCommand::Disable { name } => (
            "core.disable_plugin",
            vec![RencodeValue::Str(name.clone())],
            BTreeMap::new(),
        ),
    }
}

async fn label_record_call(
    client: &DelugeClient,
    cmd: &LabelCommand,
) -> anyhow::Result<(
    String,
    Vec<RencodeValue>,
    BTreeMap<RencodeValue, RencodeValue>,
    RencodeValue,
)> {
    use BTreeMap;
    use RencodeValue;

    let (method, rpc_args): (&str, Vec<RencodeValue>) = match cmd {
        LabelCommand::List => ("label.get_labels", vec![]),
        LabelCommand::Add { id } => ("label.add", vec![RencodeValue::Str(id.clone())]),
        LabelCommand::Remove { id } => ("label.remove", vec![RencodeValue::Str(id.clone())]),
    };

    let request = DelugeRpcRequest::new(method)
        .with_args(rpc_args.clone())
        .with_kwargs(BTreeMap::new());
    let response = client.call(request).await?;

    Ok((method.to_owned(), rpc_args, BTreeMap::new(), response))
}

fn parse_record_keys(keys: &Option<String>) -> Vec<String> {
    match keys {
        Some(k) => {
            let json: Result<Vec<String>, _> = serde_json::from_str(k);
            json.unwrap_or_default()
        }
        None => vec![],
    }
}
