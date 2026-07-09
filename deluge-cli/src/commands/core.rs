use crate::helpers::{rencode_from_json_value, rencode_to_plain_json};
use clap::Subcommand;
use deluge_rpc_client::models::{AddTorrentOptions, FilterDict, SetTorrentOptions, TrackerInfo};
use deluge_rpc_client::{
    CoreAccountRpc, CoreConfigRpc, CoreMiscRpc, CorePluginRpc, CoreSessionRpc, CoreTorrentRpc,
    DelugeClient,
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
    /// `core.*` misc methods - create torrent, glob, completion paths.
    #[command(subcommand)]
    Misc(CoreMiscCommand),
    /// `core.*` account methods - list, create, update, remove.
    #[command(subcommand)]
    Accounts(CoreAccountsCommand),
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
            CoreCommand::Misc(sub) => sub.run(client).await,
            CoreCommand::Accounts(sub) => sub.run(client).await,
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
    /// Add a torrent from a magnet URI.
    #[command(name = "add-magnet")]
    AddMagnet {
        /// Magnet URI.
        uri: String,
    },
    /// Add a torrent from a .torrent file (base64-encoded).
    #[command(name = "add-file")]
    AddFile {
        /// Filename for the torrent.
        filename: String,
        /// Base64-encoded torrent file data.
        filedump: String,
    },
    /// Add a torrent from a .torrent file asynchronously.
    #[command(name = "add-file-async")]
    AddFileAsync {
        filename: String,
        filedump: String,
        /// Whether to save session state after adding.
        #[arg(long, default_value_t = true)]
        save_state: bool,
    },
    /// Add multiple torrents at once.
    #[command(name = "add-files")]
    AddFiles {
        /// JSON array of [filename, filedump, options] tuples.
        json: String,
    },
    /// Add a torrent from a URL.
    #[command(name = "add-url")]
    AddUrl { url: String },
    /// Prefetch magnet metadata without adding the torrent.
    #[command(name = "prefetch")]
    Prefetch {
        magnet: String,
        /// Timeout in seconds.
        #[arg(long)]
        timeout: Option<i64>,
    },
    /// Remove multiple torrents.
    #[command(name = "remove-torrents")]
    RemoveTorrents {
        /// JSON array of torrent IDs.
        ids: String,
        /// Whether to remove downloaded data.
        #[arg(long, default_value_t = true)]
        remove_data: bool,
    },
    /// Pause a single torrent.
    #[command(name = "pause")]
    Pause { torrent_id: String },
    /// Pause multiple torrents. Omit IDs to pause all.
    #[command(name = "pause-all")]
    PauseAll,
    /// Resume a single torrent.
    #[command(name = "resume")]
    Resume { torrent_id: String },
    /// Resume multiple torrents. Omit IDs to resume all.
    #[command(name = "resume-all")]
    ResumeAll,
    /// Force tracker reannounce for torrents.
    #[command(name = "reannounce")]
    Reannounce { ids: String },
    /// Force data recheck for torrents.
    #[command(name = "recheck")]
    Recheck { ids: String },
    /// Set per-torrent options.
    #[command(name = "set-options")]
    SetOptions {
        ids: String,
        /// JSON object of options.
        json: String,
    },
    /// Manually connect to a peer for a torrent.
    #[command(name = "connect-peer")]
    ConnectPeer {
        torrent_id: String,
        ip: String,
        port: i64,
    },
    /// Move downloaded data to a new location.
    #[command(name = "move-storage")]
    MoveStorage { ids: String, dest: String },
    /// Set SSL certificates for a torrent.
    #[command(name = "set-ssl-cert")]
    SetSslCert {
        torrent_id: String,
        certificate: String,
        private_key: String,
        dh_params: String,
        #[arg(long, default_value_t = true)]
        save_to_disk: bool,
    },
    /// Get the filter tree for sidebar UIs.
    #[command(name = "filter-tree")]
    FilterTree {
        #[arg(long, default_value_t = true)]
        show_zero_hits: bool,
    },
    /// Get all torrent IDs in the session.
    #[command(name = "session-state")]
    SessionState,
    /// Get the size of a file or directory.
    #[command(name = "path-size")]
    PathSize { path: String },
    /// Set trackers for a torrent.
    #[command(name = "set-trackers")]
    SetTrackers {
        torrent_id: String,
        /// JSON array of {url, tier} objects.
        json: String,
    },
    /// Rename files within a torrent.
    #[command(name = "rename-files")]
    RenameFiles {
        torrent_id: String,
        /// JSON array of [index, new_name] tuples.
        json: String,
    },
    /// Rename a folder within a torrent.
    #[command(name = "rename-folder")]
    RenameFolder {
        torrent_id: String,
        folder: String,
        new_folder: String,
    },
    /// Move torrents to top of queue.
    #[command(name = "queue-top")]
    QueueTop { ids: String },
    /// Move torrents up in queue.
    #[command(name = "queue-up")]
    QueueUp { ids: String },
    /// Move torrents down in queue.
    #[command(name = "queue-down")]
    QueueDown { ids: String },
    /// Move torrents to bottom of queue.
    #[command(name = "queue-bottom")]
    QueueBottom { ids: String },
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
            CoreTorrentsCommand::AddMagnet { uri } => {
                let options = AddTorrentOptions::default();
                let result = client
                    .core()
                    .torrents
                    .add_torrent_magnet(uri, &options)
                    .await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreTorrentsCommand::AddFile { filename, filedump } => {
                let options = AddTorrentOptions::default();
                let result = client
                    .core()
                    .torrents
                    .add_torrent_file(filename, filedump, &options)
                    .await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreTorrentsCommand::AddFileAsync {
                filename,
                filedump,
                save_state,
            } => {
                let options = AddTorrentOptions::default();
                let result = client
                    .core()
                    .torrents
                    .add_torrent_file_async(filename, filedump, &options, *save_state)
                    .await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreTorrentsCommand::AddFiles { json } => {
                let parsed: Vec<(String, String, AddTorrentOptions)> =
                    serde_json::from_str(json)
                        .map_err(|e| anyhow::anyhow!("failed to parse torrent files JSON: {e}"))?;
                let result = client.core().torrents.add_torrent_files(&parsed).await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreTorrentsCommand::AddUrl { url } => {
                let options = AddTorrentOptions::default();
                let result = client
                    .core()
                    .torrents
                    .add_torrent_url(url, &options, None)
                    .await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreTorrentsCommand::Prefetch { magnet, timeout } => {
                let result = client
                    .core()
                    .torrents
                    .prefetch_magnet_metadata(magnet, *timeout)
                    .await?;
                let mut map = serde_json::Map::new();
                map.insert(
                    "torrent_id".to_owned(),
                    JsonValue::from(result.torrent_id.as_str()),
                );
                map.insert(
                    "metadata_len".to_owned(),
                    JsonValue::from(result.metadata.len()),
                );
                Ok(serde_json::to_string_pretty(&JsonValue::Object(map))?)
            }
            CoreTorrentsCommand::RemoveTorrents { ids, remove_data } => {
                let torrent_ids: Vec<String> = serde_json::from_str(ids)
                    .map_err(|e| anyhow::anyhow!("failed to parse torrent IDs JSON: {e}"))?;
                let result = client
                    .core()
                    .torrents
                    .remove_torrents(&torrent_ids, *remove_data)
                    .await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreTorrentsCommand::Pause { torrent_id } => {
                client.core().torrents.pause_torrent(torrent_id).await?;
                Ok("Torrent paused.".to_owned())
            }
            CoreTorrentsCommand::PauseAll => {
                client.core().torrents.pause_torrents(None).await?;
                Ok("All torrents paused.".to_owned())
            }
            CoreTorrentsCommand::Resume { torrent_id } => {
                client.core().torrents.resume_torrent(torrent_id).await?;
                Ok("Torrent resumed.".to_owned())
            }
            CoreTorrentsCommand::ResumeAll => {
                client.core().torrents.resume_torrents(None).await?;
                Ok("All torrents resumed.".to_owned())
            }
            CoreTorrentsCommand::Reannounce { ids } => {
                let torrent_ids: Vec<String> = serde_json::from_str(ids)
                    .map_err(|e| anyhow::anyhow!("failed to parse torrent IDs JSON: {e}"))?;
                client
                    .core()
                    .torrents
                    .force_reannounce(&torrent_ids)
                    .await?;
                Ok("Reannounce sent.".to_owned())
            }
            CoreTorrentsCommand::Recheck { ids } => {
                let torrent_ids: Vec<String> = serde_json::from_str(ids)
                    .map_err(|e| anyhow::anyhow!("failed to parse torrent IDs JSON: {e}"))?;
                client.core().torrents.force_recheck(&torrent_ids).await?;
                Ok("Recheck sent.".to_owned())
            }
            CoreTorrentsCommand::SetOptions { ids, json } => {
                let torrent_ids: Vec<String> = serde_json::from_str(ids)
                    .map_err(|e| anyhow::anyhow!("failed to parse torrent IDs JSON: {e}"))?;
                let options: SetTorrentOptions = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse options JSON: {e}"))?;
                client
                    .core()
                    .torrents
                    .set_torrent_options(&torrent_ids, &options)
                    .await?;
                Ok("Options set.".to_owned())
            }
            CoreTorrentsCommand::ConnectPeer {
                torrent_id,
                ip,
                port,
            } => {
                client
                    .core()
                    .torrents
                    .connect_peer(torrent_id, ip, *port)
                    .await?;
                Ok("Peer connection requested.".to_owned())
            }
            CoreTorrentsCommand::MoveStorage { ids, dest } => {
                let torrent_ids: Vec<String> = serde_json::from_str(ids)
                    .map_err(|e| anyhow::anyhow!("failed to parse torrent IDs JSON: {e}"))?;
                client
                    .core()
                    .torrents
                    .move_storage(&torrent_ids, dest)
                    .await?;
                Ok("Storage moved.".to_owned())
            }
            CoreTorrentsCommand::SetSslCert {
                torrent_id,
                certificate,
                private_key,
                dh_params,
                save_to_disk,
            } => {
                client
                    .core()
                    .torrents
                    .set_ssl_torrent_cert(
                        torrent_id,
                        certificate,
                        private_key,
                        dh_params,
                        *save_to_disk,
                    )
                    .await?;
                Ok("SSL cert set.".to_owned())
            }
            CoreTorrentsCommand::FilterTree { show_zero_hits } => {
                let result = client
                    .core()
                    .torrents
                    .get_filter_tree(*show_zero_hits, None)
                    .await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreTorrentsCommand::SessionState => {
                let result = client.core().torrents.get_session_state().await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreTorrentsCommand::PathSize { path } => {
                let result = client.core().torrents.get_path_size(path).await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreTorrentsCommand::SetTrackers { torrent_id, json } => {
                let trackers: Vec<TrackerInfo> = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse trackers JSON: {e}"))?;
                client
                    .core()
                    .torrents
                    .set_torrent_trackers(torrent_id, &trackers)
                    .await?;
                Ok("Trackers set.".to_owned())
            }
            CoreTorrentsCommand::RenameFiles { torrent_id, json } => {
                let filenames: Vec<(i64, String)> = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse filenames JSON: {e}"))?;
                client
                    .core()
                    .torrents
                    .rename_files(torrent_id, &filenames)
                    .await?;
                Ok("Files renamed.".to_owned())
            }
            CoreTorrentsCommand::RenameFolder {
                torrent_id,
                folder,
                new_folder,
            } => {
                client
                    .core()
                    .torrents
                    .rename_folder(torrent_id, folder, new_folder)
                    .await?;
                Ok("Folder renamed.".to_owned())
            }
            CoreTorrentsCommand::QueueTop { ids } => {
                let torrent_ids: Vec<String> = serde_json::from_str(ids)
                    .map_err(|e| anyhow::anyhow!("failed to parse torrent IDs JSON: {e}"))?;
                client.core().torrents.queue_top(&torrent_ids).await?;
                Ok("Queue top.".to_owned())
            }
            CoreTorrentsCommand::QueueUp { ids } => {
                let torrent_ids: Vec<String> = serde_json::from_str(ids)
                    .map_err(|e| anyhow::anyhow!("failed to parse torrent IDs JSON: {e}"))?;
                client.core().torrents.queue_up(&torrent_ids).await?;
                Ok("Queue up.".to_owned())
            }
            CoreTorrentsCommand::QueueDown { ids } => {
                let torrent_ids: Vec<String> = serde_json::from_str(ids)
                    .map_err(|e| anyhow::anyhow!("failed to parse torrent IDs JSON: {e}"))?;
                client.core().torrents.queue_down(&torrent_ids).await?;
                Ok("Queue down.".to_owned())
            }
            CoreTorrentsCommand::QueueBottom { ids } => {
                let torrent_ids: Vec<String> = serde_json::from_str(ids)
                    .map_err(|e| anyhow::anyhow!("failed to parse torrent IDs JSON: {e}"))?;
                client.core().torrents.queue_bottom(&torrent_ids).await?;
                Ok("Queue bottom.".to_owned())
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
    /// Pause the entire session (all torrents).
    #[command(name = "pause")]
    PauseSession,
    /// Resume the entire session.
    #[command(name = "resume")]
    ResumeSession,
    /// Returns whether the session is paused.
    #[command(name = "is-paused")]
    IsSessionPaused,
    /// Returns the active listen port for incoming connections.
    #[command(name = "listen-port")]
    ListenPort,
    /// Returns the active SSL listen port.
    #[command(name = "ssl-listen-port")]
    SslListenPort,
    /// Returns the external IP address as determined by libtorrent.
    #[command(name = "external-ip")]
    ExternalIp,
    /// Returns the libtorrent version string.
    #[command(name = "libtorrent-version")]
    LibtorrentVersion,
    /// Tests whether the active listen port is open. May be slow (network-dependent).
    #[command(name = "test-listen-port")]
    TestListenPort,
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
            CoreSessionCommand::PauseSession => {
                client.core().session.pause_session().await?;
                Ok("Session paused.".to_owned())
            }
            CoreSessionCommand::ResumeSession => {
                client.core().session.resume_session().await?;
                Ok("Session resumed.".to_owned())
            }
            CoreSessionCommand::IsSessionPaused => {
                let paused = client.core().session.is_session_paused().await?;
                Ok(serde_json::to_string_pretty(&paused)?)
            }
            CoreSessionCommand::ListenPort => {
                let port = client.core().session.get_listen_port().await?;
                Ok(serde_json::to_string_pretty(&port)?)
            }
            CoreSessionCommand::SslListenPort => {
                let port = client.core().session.get_ssl_listen_port().await?;
                Ok(serde_json::to_string_pretty(&port)?)
            }
            CoreSessionCommand::ExternalIp => {
                let ip = client.core().session.get_external_ip().await?;
                Ok(serde_json::to_string_pretty(&ip)?)
            }
            CoreSessionCommand::LibtorrentVersion => {
                let version = client.core().session.get_libtorrent_version().await?;
                Ok(serde_json::to_string_pretty(&version)?)
            }
            CoreSessionCommand::TestListenPort => {
                let result = client.core().session.test_listen_port().await?;
                Ok(serde_json::to_string_pretty(&result)?)
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
    /// Get a subset of config values by keys.
    GetValues {
        /// JSON array of config keys to fetch.
        keys: String,
    },
    /// Set config values from a JSON object. Keys in `read_only_config_keys`
    /// are skipped. Type coercion is enforced - mismatched types raise an error.
    Set {
        /// JSON object of config key-value pairs.
        json: String,
    },
    /// Returns live proxy settings from the libtorrent session.
    Proxy,
}

impl CoreConfigCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            CoreConfigCommand::Get { key } => match key {
                Some(k) => {
                    let value = client.core().config.get_config_value(k).await?;
                    let tagged = deluge_rpc_rencode::to_json(&value);
                    Ok(serde_json::to_string_pretty(&tagged)?)
                }
                None => {
                    let config = client.core().config.get_config().await?;
                    Ok(serde_json::to_string_pretty(&config)?)
                }
            },
            CoreConfigCommand::GetValues { keys } => {
                let keys_list: Vec<String> = serde_json::from_str(keys)
                    .map_err(|e| anyhow::anyhow!("failed to parse keys JSON: {e}"))?;
                let values = client.core().config.get_config_values(&keys_list).await?;
                let mut map = serde_json::Map::new();
                for (k, v) in &values {
                    map.insert(k.clone(), rencode_to_plain_json(v));
                }
                Ok(serde_json::to_string_pretty(&JsonValue::Object(map))?)
            }
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
            CoreConfigCommand::Proxy => {
                let proxy = client.core().config.get_proxy().await?;
                Ok(serde_json::to_string_pretty(&proxy)?)
            }
        }
    }
}

/// `core.*` plugin management methods.
#[derive(Subcommand, Debug, Clone)]
pub enum PluginsListCommand {
    /// List names of currently enabled plugins.
    List,
    /// List names of all available plugins (installed but not necessarily enabled).
    #[command(name = "available")]
    Available,
    /// Enable a plugin. Returns `true` on success or if already enabled.
    Enable { name: String },
    /// Disable a plugin. Returns `true` on success or if already disabled.
    Disable { name: String },
    /// Upload and install a new plugin from base64-encoded data.
    #[command(name = "upload")]
    Upload { filename: String, filedump: String },
    /// Rescan plugin folders for newly installed plugins.
    #[command(name = "rescan")]
    Rescan,
}

impl PluginsListCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            PluginsListCommand::List => {
                let enabled = client.core().plugins.get_enabled_plugins().await?;
                Ok(serde_json::to_string_pretty(&enabled)?)
            }
            PluginsListCommand::Available => {
                let available = client.core().plugins.get_available_plugins().await?;
                Ok(serde_json::to_string_pretty(&available)?)
            }
            PluginsListCommand::Enable { name } => {
                let result = client.core().plugins.enable_plugin(name).await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            PluginsListCommand::Disable { name } => {
                let result = client.core().plugins.disable_plugin(name).await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            PluginsListCommand::Upload { filename, filedump } => {
                client
                    .core()
                    .plugins
                    .upload_plugin(filename, filedump)
                    .await?;
                Ok("Plugin uploaded.".to_owned())
            }
            PluginsListCommand::Rescan => {
                client.core().plugins.rescan_plugins().await?;
                Ok("Plugins rescanned.".to_owned())
            }
        }
    }
}

/// Default piece length for torrent creation (256 KiB).
const DEFAULT_PIECE_LENGTH: i64 = 262144;

/// `core.*` misc methods.
#[derive(Subcommand, Debug, Clone)]
pub enum CoreMiscCommand {
    /// Glob filesystem paths matching a pattern.
    Glob {
        /// Glob pattern to match.
        path: String,
    },
    /// Get path completions for a partial path input.
    CompletionPaths {
        /// Partial path text to complete.
        text: String,
        /// Whether to include hidden files.
        #[arg(long, default_value_t = false)]
        show_hidden: bool,
    },
    /// Create a torrent file from a path.
    #[command(name = "create-torrent")]
    CreateTorrent {
        /// Path to the file or directory to create a torrent from.
        path: String,
        /// Tracker URL.
        tracker: String,
        /// Piece length in bytes.
        #[arg(long)]
        piece_length: Option<i64>,
    },
}

impl CoreMiscCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            CoreMiscCommand::Glob { path } => {
                let result = client.core().misc.glob(path).await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreMiscCommand::CompletionPaths { text, show_hidden } => {
                let result = client
                    .core()
                    .misc
                    .get_completion_paths(text, *show_hidden)
                    .await?;
                let mut map = serde_json::Map::new();
                map.insert(
                    "completion_text".to_owned(),
                    JsonValue::from(result.completion_text.as_str()),
                );
                map.insert(
                    "show_hidden_files".to_owned(),
                    JsonValue::from(result.show_hidden_files),
                );
                map.insert(
                    "paths".to_owned(),
                    JsonValue::Array(result.paths.into_iter().map(JsonValue::from).collect()),
                );
                Ok(serde_json::to_string_pretty(&JsonValue::Object(map))?)
            }
            CoreMiscCommand::CreateTorrent {
                path,
                tracker,
                piece_length,
            } => {
                let result = client
                    .core()
                    .misc
                    .create_torrent(
                        path,
                        tracker,
                        piece_length.unwrap_or(DEFAULT_PIECE_LENGTH),
                        None,
                        None,
                        None,
                        false,
                        None,
                        None,
                        false,
                    )
                    .await?;
                let mut map = serde_json::Map::new();
                map.insert(
                    "filename".to_owned(),
                    JsonValue::from(result.filename.as_str()),
                );
                map.insert(
                    "file_dump".to_owned(),
                    JsonValue::from(result.file_dump.as_str()),
                );
                Ok(serde_json::to_string_pretty(&JsonValue::Object(map))?)
            }
        }
    }
}

/// `core.*` account management methods.
#[derive(Subcommand, Debug, Clone)]
pub enum CoreAccountsCommand {
    /// List all known user accounts.
    #[command(name = "list")]
    List,
    /// Create a new user account.
    #[command(name = "create")]
    Create {
        username: String,
        password: String,
        auth_level: String,
    },
    /// Update an existing account's password and/or auth level.
    #[command(name = "update")]
    Update {
        username: String,
        password: String,
        auth_level: String,
    },
    /// Remove a user account.
    #[command(name = "remove")]
    Remove { username: String },
    /// Get auth level name-to-int and int-to-name mappings.
    #[command(name = "mappings")]
    Mappings,
}

impl CoreAccountsCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            CoreAccountsCommand::List => {
                let accounts = client.core().accounts.get_known_accounts().await?;
                let arr: Vec<JsonValue> = accounts
                    .iter()
                    .map(|a| {
                        let mut m = serde_json::Map::new();
                        m.insert("username".to_owned(), JsonValue::from(a.username.as_str()));
                        m.insert(
                            "auth_level".to_owned(),
                            JsonValue::from(a.auth_level.as_str()),
                        );
                        m.insert(
                            "auth_level_int".to_owned(),
                            JsonValue::from(a.auth_level_int),
                        );
                        JsonValue::Object(m)
                    })
                    .collect();
                Ok(serde_json::to_string_pretty(&JsonValue::Array(arr))?)
            }
            CoreAccountsCommand::Create {
                username,
                password,
                auth_level,
            } => {
                let result = client
                    .core()
                    .accounts
                    .create_account(username, password, auth_level)
                    .await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreAccountsCommand::Update {
                username,
                password,
                auth_level,
            } => {
                let result = client
                    .core()
                    .accounts
                    .update_account(username, password, auth_level)
                    .await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreAccountsCommand::Remove { username } => {
                let result = client.core().accounts.remove_account(username).await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            CoreAccountsCommand::Mappings => {
                let (name_to_int, int_to_name) =
                    client.core().accounts.get_auth_levels_mappings().await?;
                let mut map = serde_json::Map::new();
                map.insert(
                    "name_to_int".to_owned(),
                    serde_json::to_value(
                        name_to_int
                            .iter()
                            .map(|(k, v)| (k.clone(), *v))
                            .collect::<BTreeMap<_, _>>(),
                    )?,
                );
                map.insert(
                    "int_to_name".to_owned(),
                    serde_json::to_value(
                        int_to_name
                            .iter()
                            .map(|(k, v)| (*k, v.clone()))
                            .collect::<BTreeMap<_, _>>(),
                    )?,
                );
                Ok(serde_json::to_string_pretty(&JsonValue::Object(map))?)
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
