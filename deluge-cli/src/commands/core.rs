use crate::commands::call::{rencode_from_json_value, rencode_to_plain_json};
use clap::Subcommand;
use deluge_rpc::models::torrents::FilterDict;
use deluge_rpc::{CoreConfigRpc, CorePluginRpc, CoreSessionRpc, CoreTorrentRpc, DelugeClient};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

#[derive(Subcommand, Debug, Clone)]
pub enum CoreCommand {
    #[command(name = "free-space")]
    FreeSpace {
        #[arg()]
        path: Option<String>,
    },

    #[command(subcommand)]
    Torrents(CoreTorrentsCommand),

    #[command(subcommand)]
    Session(CoreSessionCommand),

    #[command(subcommand)]
    Config(CoreConfigCommand),

    #[command(subcommand)]
    Plugins(PluginsListCommand),
}

#[derive(Subcommand, Debug, Clone)]
pub enum CoreTorrentsCommand {
    List {
        #[arg(long)]
        filter: Option<String>,

        #[arg(long)]
        keys: Option<String>,
    },

    Status {
        torrent_id: String,

        #[arg(long)]
        keys: Option<String>,
    },

    Remove {
        torrent_id: String,

        #[arg(long)]
        keep_data: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum CoreSessionCommand {
    Status {
        #[arg(long)]
        keys: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum CoreConfigCommand {
    Get { key: Option<String> },
    Set { json: String },
}

#[derive(Subcommand, Debug, Clone)]
pub enum PluginsListCommand {
    List,
    Enable { name: String },
    Disable { name: String },
}

pub async fn run_core(client: &DelugeClient, cmd: &CoreCommand) -> anyhow::Result<String> {
    match cmd {
        CoreCommand::FreeSpace { path } => {
            let space = client.core().session.get_free_space(path.clone()).await?;
            Ok(serde_json::to_string_pretty(&space)?)
        }
        CoreCommand::Torrents(sub) => run_core_torrents(client, sub).await,
        CoreCommand::Session(sub) => run_core_session(client, sub).await,
        CoreCommand::Config(sub) => run_core_config(client, sub).await,
        CoreCommand::Plugins(sub) => run_core_plugins(client, sub).await,
    }
}

async fn run_core_torrents(
    client: &DelugeClient,
    cmd: &CoreTorrentsCommand,
) -> anyhow::Result<String> {
    match cmd {
        CoreTorrentsCommand::List { filter, keys } => {
            let filter_dict = parse_filter_dict(filter)?;
            let keys_list = parse_keys(keys);
            let entries = client
                .core()
                .torrents
                .get_torrents_status(&filter_dict, &keys_list, false)
                .await?;
            Ok(serde_json::to_string_pretty(&entries)?)
        }
        CoreTorrentsCommand::Status { torrent_id, keys } => {
            let keys_list = parse_keys(keys);
            let status = client
                .core()
                .torrents
                .get_torrent_status(torrent_id, &keys_list, false)
                .await?;
            Ok(serde_json::to_string_pretty(&status)?)
        }
        CoreTorrentsCommand::Remove {
            torrent_id,
            keep_data,
        } => {
            let result = client
                .core()
                .torrents
                .remove_torrent(torrent_id, !keep_data)
                .await?;
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

async fn run_core_session(
    client: &DelugeClient,
    cmd: &CoreSessionCommand,
) -> anyhow::Result<String> {
    match cmd {
        CoreSessionCommand::Status { keys } => {
            let keys_list = parse_keys(keys);
            let status = client.core().session.get_session_status(&keys_list).await?;

            let mut map = serde_json::Map::new();
            map.insert("download_rate".to_owned(), JsonValue::from(status.download_rate));
            map.insert("upload_rate".to_owned(), JsonValue::from(status.upload_rate));
            map.insert(
                "payload_download_rate".to_owned(),
                JsonValue::from(status.payload_download_rate),
            );
            map.insert(
                "payload_upload_rate".to_owned(),
                JsonValue::from(status.payload_upload_rate),
            );
            map.insert(
                "ip_overhead_download_rate".to_owned(),
                JsonValue::from(status.ip_overhead_download_rate),
            );
            map.insert(
                "ip_overhead_upload_rate".to_owned(),
                JsonValue::from(status.ip_overhead_upload_rate),
            );
            map.insert(
                "tracker_download_rate".to_owned(),
                JsonValue::from(status.tracker_download_rate),
            );
            map.insert(
                "tracker_upload_rate".to_owned(),
                JsonValue::from(status.tracker_upload_rate),
            );
            map.insert(
                "dht_download_rate".to_owned(),
                JsonValue::from(status.dht_download_rate),
            );
            map.insert(
                "dht_upload_rate".to_owned(),
                JsonValue::from(status.dht_upload_rate),
            );
            map.insert(
                "write_hit_ratio".to_owned(),
                JsonValue::from(status.write_hit_ratio),
            );
            map.insert(
                "read_hit_ratio".to_owned(),
                JsonValue::from(status.read_hit_ratio),
            );
            let mut extra_keys: Vec<&String> = status.extra.keys().collect();
            extra_keys.sort();
            for k in extra_keys {
                map.insert(k.clone(), rencode_to_plain_json(&status.extra[k]));
            }

            Ok(serde_json::to_string_pretty(&JsonValue::Object(map))?)
        }
    }
}

async fn run_core_config(client: &DelugeClient, cmd: &CoreConfigCommand) -> anyhow::Result<String> {
    match cmd {
        CoreConfigCommand::Get { key } => match key {
            Some(k) => {
                let value = client.core().config.get_config_value(k).await?;
                let tagged = deluge_rpc::to_json(&value);
                Ok(serde_json::to_string_pretty(&tagged)?)
            }
            None => {
                let config = client.core().config.get_config().await?;
                Ok(serde_json::to_string_pretty(&config)?)
            }
        },
        CoreConfigCommand::Set { json } => {
            let parsed: JsonValue = serde_json::from_str(json)
                .map_err(|e| anyhow::anyhow!("failed to parse config JSON: {e}"))?;
            let obj = parsed
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("config set value must be a JSON object"))?;
            let mut config = BTreeMap::new();
            for (k, v) in obj {
                let rencode_val = rencode_from_json_value(v)?;
                config.insert(k.clone(), rencode_val);
            }
            client.core().config.set_config(&config).await?;
            Ok("Config updated.".to_owned())
        }
    }
}

async fn run_core_plugins(
    client: &DelugeClient,
    cmd: &PluginsListCommand,
) -> anyhow::Result<String> {
    match cmd {
        PluginsListCommand::List => {
            let enabled = client.core().plugins.get_enabled_plugins().await?;
            Ok(serde_json::to_string_pretty(&enabled)?)
        }
        PluginsListCommand::Enable { name } => {
            let result = client.core().plugins.enable_plugin(name).await?;
            Ok(serde_json::to_string_pretty(&result)?)
        }
        PluginsListCommand::Disable { name } => {
            let result = client.core().plugins.disable_plugin(name).await?;
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

fn parse_filter_dict(filter: &Option<String>) -> anyhow::Result<FilterDict> {
    match filter {
        Some(f) => {
            let json: JsonValue = serde_json::from_str(f)
                .map_err(|e| anyhow::anyhow!("failed to parse filter JSON: {e}"))?;
            let filter_dict: FilterDict = serde_json::from_value(json)
                .map_err(|e| anyhow::anyhow!("failed to deserialize filter dict: {e}"))?;
            Ok(filter_dict)
        }
        None => Ok(Default::default()),
    }
}

fn parse_keys(keys: &Option<String>) -> Vec<String> {
    match keys {
        Some(k) => {
            let json: Result<Vec<String>, _> = serde_json::from_str(k);
            json.unwrap_or_else(|_| vec![])
        }
        None => vec![],
    }
}
