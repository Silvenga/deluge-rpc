use crate::helpers::{rencode_from_json_value, rencode_to_plain_json};
use clap::Subcommand;
use deluge_rpc_client::models::FilterDict;
use deluge_rpc_client::{
    CoreConfigRpc, CorePluginRpc, CoreSessionRpc, CoreTorrentRpc, DelugeClient,
};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

/// `core.*` methods - torrents, session, config, and plugins.
#[derive(Subcommand, Debug, Clone)]
pub enum CoreCommand {
    /// Get free space in bytes at a path. `None` uses the default download location.
    /// Negative on error.
    #[command(name = "free-space")]
    FreeSpace {
        #[arg()]
        path: Option<String>,
    },
    /// `core.*` torrent methods - list, status, remove.
    #[command(subcommand)]
    Torrents(CoreTorrentsCommand),
    /// `core.*` session methods - session status.
    #[command(subcommand)]
    Session(CoreSessionCommand),
    /// `core.*` config methods - get and set daemon config.`daemon.*` method
    #[command(subcommand)]
    Config(CoreConfigCommand),
    /// `core.*` plugin methods - list, enable, disable.
    #[command(subcommand)]
    Plugins(PluginsListCommand),
}

impl CoreCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            CoreCommand::FreeSpace { path } => {
                let space = client.core().session.get_free_space(path.clone()).await?;
                Ok(serde_json::to_string_pretty(&space)?)
            }
            CoreCommand::Torrents(sub) => sub.run(client).await,
            CoreCommand::Session(sub) => sub.run(client).await,
            CoreCommand::Config(sub) => sub.run(client).await,
            CoreCommand::Plugins(sub) => sub.run(client).await,
        }
    }
}

/// `core.*` torrent methods.
#[derive(Subcommand, Debug, Clone)]
pub enum CoreTorrentsCommand {
    /// List torrents matching a filter dict. `filter_dict={}` returns all.
    /// `keys=[]` returns all status keys. See SPEC-core-torrents.md "Filter dict".
    List {
        /// JSON filter dict (e.g. `{"state":["Downloading"]}`). `{}` = no filter.
        #[arg(long)]
        filter: Option<String>,
        /// JSON array of status keys to return. `[]` = all keys.
        #[arg(long)]
        keys: Option<String>,
    },
    /// Get status values for a single torrent. `keys=[]` returns all keys.
    /// See SPEC-core-torrents.md "Torrent status keys".
    Status {
        torrent_id: String,
        /// JSON array of status keys to return. `[]` = all keys.
        #[arg(long)]
        keys: Option<String>,
    },
    /// Remove a torrent. `keep_data=true` preserves downloaded files.
    /// Raises `InvalidTorrentError` if the torrent_id is not found.
    Remove {
        torrent_id: String,
        /// Whether to keep the downloaded data (default: remove data).
        #[arg(long)]
        keep_data: bool,
    },
}

impl CoreTorrentsCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
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
}

/// `core.*` session methods.
#[derive(Subcommand, Debug, Clone)]
pub enum CoreSessionCommand {
    /// Get libtorrent session statistics for the requested keys.
    /// `keys=[]` returns all keys. See SPEC-core-session.md "Session status keys".
    Status {
        /// JSON array of session status keys to return. `[]` = all keys.
        #[arg(long)]
        keys: Option<String>,
    },
}

impl CoreSessionCommand {
    async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            CoreSessionCommand::Status { keys } => {
                let keys_list = parse_keys(keys);
                let status = client.core().session.get_session_status(&keys_list).await?;

                let mut map = serde_json::Map::new();
                map.insert(
                    "download_rate".to_owned(),
                    JsonValue::from(status.download_rate),
                );
                map.insert(
                    "upload_rate".to_owned(),
                    JsonValue::from(status.upload_rate),
                );
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
                let mut extra: Vec<_> = status.extra.iter().collect();
                extra.sort_by_key(|(k, _)| *k);
                for (k, v) in extra {
                    map.insert(k.clone(), rencode_to_plain_json(v));
                }

                Ok(serde_json::to_string_pretty(&JsonValue::Object(map))?)
            }
        }
    }
}

/// `core.*` config methods - get and set daemon config preferences.
#[derive(Subcommand, Debug, Clone)]
pub enum CoreConfigCommand {
    /// Get config values. With no key, returns all config preferences.
    /// With a key, returns a single config value (`null` for unknown keys).
    /// See SPEC-core-session.md "Config keys".
    Get { key: Option<String> },
    /// Set config values from a JSON object. Keys in `read_only_config_keys`
    /// are skipped. Type coercion is enforced - mismatched types raise an error.
    Set {
        /// JSON object of config key-value pairs.
        json: String,
    },
}

impl CoreConfigCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            CoreConfigCommand::Get { key } => match key {
                Some(k) => {
                    let value = client.core().config.get_config_value(k).await?;
                    let tagged = deluge_rpc_client::to_json(&value);
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
}

/// `core.*` plugin management methods.
#[derive(Subcommand, Debug, Clone)]
pub enum PluginsListCommand {
    /// List names of currently enabled plugins.
    List,
    /// Enable a plugin. Returns `true` on success or if already enabled.
    Enable {
        /// Plugin name to enable.
        name: String,
    },
    /// Disable a plugin. Returns `true` on success or if already disabled.
    Disable {
        /// Plugin name to disable.
        name: String,
    },
}

impl PluginsListCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
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
