use crate::client::RpcCaller;
use crate::models::torrents::{
    AddTorrentFileResult, AddTorrentFilesResult, AddTorrentOptions, FilterDict, FilterTree,
    GetMagnetUriResult, PrefetchMagnetResult, RemoveTorrentsResult, SetTorrentOptions,
    TorrentEntry, TorrentStatus, TrackerInfo,
};
use crate::protocol::DelugeRpcRequest;
use crate::protocol::{extract_single, extract_single_dict, extract_single_int};
use crate::rencode::{RencodeValue, to_rencode_value};
use crate::shared::Shared;
use crate::transport::DelugeWriter;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait CoreTorrentRpc: Send + Sync {
    // Add / remove
    async fn add_torrent_file(
        &self, filename: &str, filedump: &str, options: &AddTorrentOptions,
    ) -> anyhow::Result<AddTorrentFileResult>;
    async fn add_torrent_file_async(
        &self, filename: &str, filedump: &str, options: &AddTorrentOptions, save_state: bool,
    ) -> anyhow::Result<AddTorrentFileResult>;
    async fn add_torrent_files(
        &self, torrent_files: &[(String, String, AddTorrentOptions)],
    ) -> anyhow::Result<AddTorrentFilesResult>;
    async fn add_torrent_url(
        &self, url: &str, options: &AddTorrentOptions,
        headers: Option<BTreeMap<String, String>>,
    ) -> anyhow::Result<AddTorrentFileResult>;
    async fn add_torrent_magnet(
        &self, uri: &str, options: &AddTorrentOptions,
    ) -> anyhow::Result<String>;
    async fn prefetch_magnet_metadata(
        &self, magnet: &str, timeout_secs: Option<i64>,
    ) -> anyhow::Result<PrefetchMagnetResult>;
    async fn remove_torrent(&self, torrent_id: &str, remove_data: bool) -> anyhow::Result<bool>;
    async fn remove_torrents(
        &self, torrent_ids: &[String], remove_data: bool,
    ) -> anyhow::Result<RemoveTorrentsResult>;

    // State control
    async fn pause_torrent(&self, torrent_id: &str) -> anyhow::Result<()>;
    async fn pause_torrents(&self, torrent_ids: Option<Vec<String>>) -> anyhow::Result<()>;
    async fn resume_torrent(&self, torrent_id: &str) -> anyhow::Result<()>;
    async fn resume_torrents(&self, torrent_ids: Option<Vec<String>>) -> anyhow::Result<()>;
    async fn force_reannounce(&self, torrent_ids: &[String]) -> anyhow::Result<()>;
    async fn force_recheck(&self, torrent_ids: &[String]) -> anyhow::Result<()>;
    async fn set_torrent_options(
        &self, torrent_ids: &[String], options: &SetTorrentOptions,
    ) -> anyhow::Result<()>;
    async fn connect_peer(&self, torrent_id: &str, ip: &str, port: i64) -> anyhow::Result<()>;
    async fn move_storage(&self, torrent_ids: &[String], dest: &str) -> anyhow::Result<()>;
    async fn set_ssl_torrent_cert(
        &self, torrent_id: &str, certificate: &str, private_key: &str, dh_params: &str,
        save_to_disk: bool,
    ) -> anyhow::Result<()>;

    // Queries
    async fn get_torrent_status(
        &self, torrent_id: &str, keys: &[String], diff: bool,
    ) -> anyhow::Result<TorrentStatus>;
    async fn get_torrents_status(
        &self, filter_dict: &FilterDict, keys: &[String], diff: bool,
    ) -> anyhow::Result<Vec<TorrentEntry>>;
    async fn get_filter_tree(
        &self, show_zero_hits: bool, hide_cat: Option<Vec<String>>,
    ) -> anyhow::Result<FilterTree>;
    async fn get_session_state(&self) -> anyhow::Result<Vec<String>>;
    async fn get_magnet_uri(&self, torrent_id: &str) -> anyhow::Result<GetMagnetUriResult>;
    async fn get_path_size(&self, path: &str) -> anyhow::Result<i64>;

    // Trackers / files / folders
    async fn set_torrent_trackers(
        &self, torrent_id: &str, trackers: &[TrackerInfo],
    ) -> anyhow::Result<()>;
    async fn rename_files(
        &self, torrent_id: &str, filenames: &[(i64, String)],
    ) -> anyhow::Result<()>;
    async fn rename_folder(
        &self, torrent_id: &str, folder: &str, new_folder: &str,
    ) -> anyhow::Result<()>;

    // Queue
    async fn queue_top(&self, torrent_ids: &[String]) -> anyhow::Result<()>;
    async fn queue_up(&self, torrent_ids: &[String]) -> anyhow::Result<()>;
    async fn queue_down(&self, torrent_ids: &[String]) -> anyhow::Result<()>;
    async fn queue_bottom(&self, torrent_ids: &[String]) -> anyhow::Result<()>;
}

pub struct CoreTorrentClient {
    caller: RpcCaller,
}

impl CoreTorrentClient {
    #[expect(dead_code, reason = "used in task 11 when connection code is updated")]
    pub(crate) fn new(shared: Arc<Shared>, writer: Arc<Mutex<DelugeWriter>>) -> Self {
        Self {
            caller: RpcCaller::new(shared, writer),
        }
    }
}

impl Clone for CoreTorrentClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl CoreTorrentRpc for CoreTorrentClient {
    // --- Add / remove ---

    async fn add_torrent_file(
        &self, filename: &str, filedump: &str, options: &AddTorrentOptions,
    ) -> anyhow::Result<AddTorrentFileResult> {
        let options_value = to_rencode_value(options).context("serializing options")?;
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.add_torrent_file").with_args(vec![
                    RencodeValue::Str(filename.to_owned()),
                    RencodeValue::Str(filedump.to_owned()),
                    options_value,
                ]),
            )
            .await
            .context("core.add_torrent_file RPC failed")?;
        let value = extract_single(&result, "core.add_torrent_file")?;
        match value {
            RencodeValue::Str(s) => Ok(Some(s)),
            RencodeValue::None => Ok(None),
            other => Err(anyhow!("core.add_torrent_file returned unexpected value: {other:?}")),
        }
    }

    async fn add_torrent_file_async(
        &self, filename: &str, filedump: &str, options: &AddTorrentOptions, save_state: bool,
    ) -> anyhow::Result<AddTorrentFileResult> {
        let options_value = to_rencode_value(options).context("serializing options")?;
        let mut kwargs = BTreeMap::new();
        kwargs.insert(RencodeValue::Str("save_state".into()), RencodeValue::Bool(save_state));
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.add_torrent_file_async")
                    .with_args(vec![
                        RencodeValue::Str(filename.to_owned()),
                        RencodeValue::Str(filedump.to_owned()),
                        options_value,
                    ])
                    .with_kwargs(kwargs),
            )
            .await
            .context("core.add_torrent_file_async RPC failed")?;
        let value = extract_single(&result, "core.add_torrent_file_async")?;
        match value {
            RencodeValue::Str(s) => Ok(Some(s)),
            RencodeValue::None => Ok(None),
            other => Err(anyhow!("core.add_torrent_file_async returned unexpected value: {other:?}")),
        }
    }

    async fn add_torrent_files(
        &self, torrent_files: &[(String, String, AddTorrentOptions)],
    ) -> anyhow::Result<AddTorrentFilesResult> {
        let mut items = Vec::with_capacity(torrent_files.len());
        for (filename, filedump, options) in torrent_files {
            let options_value = to_rencode_value(options).context("serializing options")?;
            items.push(RencodeValue::List(vec![
                RencodeValue::Str(filename.clone()),
                RencodeValue::Str(filedump.clone()),
                options_value,
            ]));
        }
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.add_torrent_files")
                    .with_args(vec![RencodeValue::List(items)]),
            )
            .await
            .context("core.add_torrent_files RPC failed")?;
        let value = extract_single(&result, "core.add_torrent_files")?;
        AddTorrentFilesResult::deserialize(&value).context("deserializing add torrent files result")
    }

    async fn add_torrent_url(
        &self, url: &str, options: &AddTorrentOptions,
        headers: Option<BTreeMap<String, String>>,
    ) -> anyhow::Result<AddTorrentFileResult> {
        let options_value = to_rencode_value(options).context("serializing options")?;
        let mut kwargs = BTreeMap::new();
        if let Some(h) = headers {
            let headers_value = to_rencode_value(&h).context("serializing headers")?;
            kwargs.insert(RencodeValue::Str("headers".into()), headers_value);
        }
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.add_torrent_url")
                    .with_args(vec![RencodeValue::Str(url.to_owned()), options_value])
                    .with_kwargs(kwargs),
            )
            .await
            .context("core.add_torrent_url RPC failed")?;
        let value = extract_single(&result, "core.add_torrent_url")?;
        match value {
            RencodeValue::Str(s) => Ok(Some(s)),
            RencodeValue::None => Ok(None),
            other => Err(anyhow!("core.add_torrent_url returned unexpected value: {other:?}")),
        }
    }

    async fn add_torrent_magnet(
        &self, uri: &str, options: &AddTorrentOptions,
    ) -> anyhow::Result<String> {
        let options_value = to_rencode_value(options).context("serializing options")?;
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.add_torrent_magnet").with_args(vec![
                    RencodeValue::Str(uri.to_owned()),
                    options_value,
                ]),
            )
            .await
            .context("core.add_torrent_magnet RPC failed")?;
        let value = extract_single(&result, "core.add_torrent_magnet")?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(anyhow!("core.add_torrent_magnet returned non-str value: {other:?}")),
        }
    }

    async fn prefetch_magnet_metadata(
        &self, magnet: &str, timeout_secs: Option<i64>,
    ) -> anyhow::Result<PrefetchMagnetResult> {
        let mut kwargs = BTreeMap::new();
        if let Some(t) = timeout_secs {
            kwargs.insert(RencodeValue::Str("timeout".into()), RencodeValue::Int(t));
        }
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.prefetch_magnet_metadata")
                    .with_args(vec![RencodeValue::Str(magnet.to_owned())])
                    .with_kwargs(kwargs),
            )
            .await
            .context("core.prefetch_magnet_metadata RPC failed")?;
        let value = extract_single(&result, "core.prefetch_magnet_metadata")?;
        PrefetchMagnetResult::deserialize(&value).context("deserializing prefetch magnet result")
    }

    async fn remove_torrent(&self, torrent_id: &str, remove_data: bool) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.remove_torrent").with_args(vec![
                    RencodeValue::Str(torrent_id.to_owned()),
                    RencodeValue::Bool(remove_data),
                ]),
            )
            .await
            .context("core.remove_torrent RPC failed")?;
        let value = extract_single(&result, "core.remove_torrent")?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!("core.remove_torrent returned non-bool value: {other:?}")),
        }
    }

    async fn remove_torrents(
        &self, torrent_ids: &[String], remove_data: bool,
    ) -> anyhow::Result<RemoveTorrentsResult> {
        let ids: Vec<RencodeValue> =
            torrent_ids.iter().map(|id| RencodeValue::Str(id.clone())).collect();
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.remove_torrents").with_args(vec![
                    RencodeValue::List(ids),
                    RencodeValue::Bool(remove_data),
                ]),
            )
            .await
            .context("core.remove_torrents RPC failed")?;
        let value = extract_single(&result, "core.remove_torrents")?;
        RemoveTorrentsResult::deserialize(&value).context("deserializing remove torrents result")
    }

    // --- State control ---

    async fn pause_torrent(&self, torrent_id: &str) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.pause_torrent")
                    .with_args(vec![RencodeValue::Str(torrent_id.to_owned())]),
            )
            .await
            .context("core.pause_torrent RPC failed")?;
        Ok(())
    }

    async fn pause_torrents(&self, torrent_ids: Option<Vec<String>>) -> anyhow::Result<()> {
        let args = match torrent_ids {
            Some(ids) => {
                let id_values: Vec<RencodeValue> =
                    ids.into_iter().map(RencodeValue::Str).collect();
                vec![RencodeValue::List(id_values)]
            }
            None => vec![RencodeValue::None],
        };
        self.caller
            .rpc_call(DelugeRpcRequest::new("core.pause_torrents").with_args(args))
            .await
            .context("core.pause_torrents RPC failed")?;
        Ok(())
    }

    async fn resume_torrent(&self, torrent_id: &str) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.resume_torrent")
                    .with_args(vec![RencodeValue::Str(torrent_id.to_owned())]),
            )
            .await
            .context("core.resume_torrent RPC failed")?;
        Ok(())
    }

    async fn resume_torrents(&self, torrent_ids: Option<Vec<String>>) -> anyhow::Result<()> {
        let args = match torrent_ids {
            Some(ids) => {
                let id_values: Vec<RencodeValue> =
                    ids.into_iter().map(RencodeValue::Str).collect();
                vec![RencodeValue::List(id_values)]
            }
            None => vec![RencodeValue::None],
        };
        self.caller
            .rpc_call(DelugeRpcRequest::new("core.resume_torrents").with_args(args))
            .await
            .context("core.resume_torrents RPC failed")?;
        Ok(())
    }

    async fn force_reannounce(&self, torrent_ids: &[String]) -> anyhow::Result<()> {
        let id_values: Vec<RencodeValue> =
            torrent_ids.iter().map(|id| RencodeValue::Str(id.clone())).collect();
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.force_reannounce")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await
            .context("core.force_reannounce RPC failed")?;
        Ok(())
    }

    async fn force_recheck(&self, torrent_ids: &[String]) -> anyhow::Result<()> {
        let id_values: Vec<RencodeValue> =
            torrent_ids.iter().map(|id| RencodeValue::Str(id.clone())).collect();
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.force_recheck")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await
            .context("core.force_recheck RPC failed")?;
        Ok(())
    }

    async fn set_torrent_options(
        &self, torrent_ids: &[String], options: &SetTorrentOptions,
    ) -> anyhow::Result<()> {
        let id_values: Vec<RencodeValue> =
            torrent_ids.iter().map(|id| RencodeValue::Str(id.clone())).collect();
        let options_value = to_rencode_value(options).context("serializing options")?;
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.set_torrent_options").with_args(vec![
                    RencodeValue::List(id_values),
                    options_value,
                ]),
            )
            .await
            .context("core.set_torrent_options RPC failed")?;
        Ok(())
    }

    async fn connect_peer(&self, torrent_id: &str, ip: &str, port: i64) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.connect_peer").with_args(vec![
                    RencodeValue::Str(torrent_id.to_owned()),
                    RencodeValue::Str(ip.to_owned()),
                    RencodeValue::Int(port),
                ]),
            )
            .await
            .context("core.connect_peer RPC failed")?;
        Ok(())
    }

    async fn move_storage(&self, torrent_ids: &[String], dest: &str) -> anyhow::Result<()> {
        let id_values: Vec<RencodeValue> =
            torrent_ids.iter().map(|id| RencodeValue::Str(id.clone())).collect();
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.move_storage").with_args(vec![
                    RencodeValue::List(id_values),
                    RencodeValue::Str(dest.to_owned()),
                ]),
            )
            .await
            .context("core.move_storage RPC failed")?;
        Ok(())
    }

    async fn set_ssl_torrent_cert(
        &self, torrent_id: &str, certificate: &str, private_key: &str, dh_params: &str,
        save_to_disk: bool,
    ) -> anyhow::Result<()> {
        let mut kwargs = BTreeMap::new();
        kwargs.insert(
            RencodeValue::Str("save_to_disk".into()),
            RencodeValue::Bool(save_to_disk),
        );
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.set_ssl_torrent_cert")
                    .with_args(vec![
                        RencodeValue::Str(torrent_id.to_owned()),
                        RencodeValue::Str(certificate.to_owned()),
                        RencodeValue::Str(private_key.to_owned()),
                        RencodeValue::Str(dh_params.to_owned()),
                    ])
                    .with_kwargs(kwargs),
            )
            .await
            .context("core.set_ssl_torrent_cert RPC failed")?;
        Ok(())
    }

    // --- Queries ---

    async fn get_torrent_status(
        &self, torrent_id: &str, keys: &[String], diff: bool,
    ) -> anyhow::Result<TorrentStatus> {
        let key_values: Vec<RencodeValue> =
            keys.iter().map(|k| RencodeValue::Str(k.clone())).collect();
        let mut kwargs = BTreeMap::new();
        kwargs.insert(RencodeValue::Str("diff".into()), RencodeValue::Bool(diff));
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.get_torrent_status")
                    .with_args(vec![
                        RencodeValue::Str(torrent_id.to_owned()),
                        RencodeValue::List(key_values),
                    ])
                    .with_kwargs(kwargs),
            )
            .await
            .context("core.get_torrent_status RPC failed")?;
        let value = extract_single(&result, "core.get_torrent_status")?;
        TorrentStatus::deserialize(&value).context("deserializing torrent status")
    }

    async fn get_torrents_status(
        &self, filter_dict: &FilterDict, keys: &[String], diff: bool,
    ) -> anyhow::Result<Vec<TorrentEntry>> {
        let filter_value = to_rencode_value(filter_dict).context("serializing filter dict")?;
        let key_values: Vec<RencodeValue> =
            keys.iter().map(|k| RencodeValue::Str(k.clone())).collect();
        let mut kwargs = BTreeMap::new();
        kwargs.insert(RencodeValue::Str("diff".into()), RencodeValue::Bool(diff));
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.get_torrents_status")
                    .with_args(vec![filter_value, RencodeValue::List(key_values)])
                    .with_kwargs(kwargs),
            )
            .await
            .context("core.get_torrents_status RPC failed")?;

        let result_dict = extract_single_dict(&result, "core.get_torrents_status")?;

        let mut entries: Vec<(String, &RencodeValue)> = result_dict
            .iter()
            .filter_map(|(k, v)| match (k, v) {
                (RencodeValue::Str(id), fields) => Some((id.clone(), fields)),
                _ => None,
            })
            .collect();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut out = Vec::with_capacity(entries.len());
        for (info_hash, fields) in entries {
            let mut entry = TorrentEntry::deserialize(fields)
                .with_context(|| format!("deserializing torrent entry `{info_hash}`"))?;
            entry.info_hash = info_hash;
            out.push(entry);
        }
        Ok(out)
    }

    async fn get_filter_tree(
        &self, show_zero_hits: bool, hide_cat: Option<Vec<String>>,
    ) -> anyhow::Result<FilterTree> {
        let mut kwargs = BTreeMap::new();
        kwargs.insert(
            RencodeValue::Str("show_zero_hits".into()),
            RencodeValue::Bool(show_zero_hits),
        );
        if let Some(cats) = hide_cat {
            let cat_values: Vec<RencodeValue> =
                cats.into_iter().map(RencodeValue::Str).collect();
            kwargs.insert(
                RencodeValue::Str("hide_cat".into()),
                RencodeValue::List(cat_values),
            );
        }
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_filter_tree").with_kwargs(kwargs))
            .await
            .context("core.get_filter_tree RPC failed")?;
        let value = extract_single(&result, "core.get_filter_tree")?;
        FilterTree::deserialize(&value).context("deserializing filter tree")
    }

    async fn get_session_state(&self) -> anyhow::Result<Vec<String>> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_session_state"))
            .await
            .context("core.get_session_state RPC failed")?;
        let value = extract_single(&result, "core.get_session_state")?;
        match value {
            RencodeValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        RencodeValue::Str(s) => out.push(s),
                        other => {
                            return Err(anyhow!(
                                "core.get_session_state returned non-str element: {other:?}"
                            ))
                        }
                    }
                }
                Ok(out)
            }
            other => Err(anyhow!(
                "core.get_session_state returned non-list value: {other:?}"
            )),
        }
    }

    async fn get_magnet_uri(&self, torrent_id: &str) -> anyhow::Result<GetMagnetUriResult> {
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.get_magnet_uri")
                    .with_args(vec![RencodeValue::Str(torrent_id.to_owned())]),
            )
            .await
            .context("core.get_magnet_uri RPC failed")?;
        let value = extract_single(&result, "core.get_magnet_uri")?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(anyhow!("core.get_magnet_uri returned non-str value: {other:?}")),
        }
    }

    async fn get_path_size(&self, path: &str) -> anyhow::Result<i64> {
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.get_path_size")
                    .with_args(vec![RencodeValue::Str(path.to_owned())]),
            )
            .await
            .context("core.get_path_size RPC failed")?;
        extract_single_int(&result, "core.get_path_size")
    }

    // --- Trackers / files / folders ---

    async fn set_torrent_trackers(
        &self, torrent_id: &str, trackers: &[TrackerInfo],
    ) -> anyhow::Result<()> {
        let tracker_values = to_rencode_value(trackers).context("serializing trackers")?;
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.set_torrent_trackers").with_args(vec![
                    RencodeValue::Str(torrent_id.to_owned()),
                    tracker_values,
                ]),
            )
            .await
            .context("core.set_torrent_trackers RPC failed")?;
        Ok(())
    }

    async fn rename_files(
        &self, torrent_id: &str, filenames: &[(i64, String)],
    ) -> anyhow::Result<()> {
        let file_values: Vec<RencodeValue> = filenames
            .iter()
            .map(|(idx, name)| {
                RencodeValue::List(vec![RencodeValue::Int(*idx), RencodeValue::Str(name.clone())])
            })
            .collect();
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.rename_files").with_args(vec![
                    RencodeValue::Str(torrent_id.to_owned()),
                    RencodeValue::List(file_values),
                ]),
            )
            .await
            .context("core.rename_files RPC failed")?;
        Ok(())
    }

    async fn rename_folder(
        &self, torrent_id: &str, folder: &str, new_folder: &str,
    ) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.rename_folder").with_args(vec![
                    RencodeValue::Str(torrent_id.to_owned()),
                    RencodeValue::Str(folder.to_owned()),
                    RencodeValue::Str(new_folder.to_owned()),
                ]),
            )
            .await
            .context("core.rename_folder RPC failed")?;
        Ok(())
    }

    // --- Queue ---

    async fn queue_top(&self, torrent_ids: &[String]) -> anyhow::Result<()> {
        let id_values: Vec<RencodeValue> =
            torrent_ids.iter().map(|id| RencodeValue::Str(id.clone())).collect();
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.queue_top")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await
            .context("core.queue_top RPC failed")?;
        Ok(())
    }

    async fn queue_up(&self, torrent_ids: &[String]) -> anyhow::Result<()> {
        let id_values: Vec<RencodeValue> =
            torrent_ids.iter().map(|id| RencodeValue::Str(id.clone())).collect();
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.queue_up")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await
            .context("core.queue_up RPC failed")?;
        Ok(())
    }

    async fn queue_down(&self, torrent_ids: &[String]) -> anyhow::Result<()> {
        let id_values: Vec<RencodeValue> =
            torrent_ids.iter().map(|id| RencodeValue::Str(id.clone())).collect();
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.queue_down")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await
            .context("core.queue_down RPC failed")?;
        Ok(())
    }

    async fn queue_bottom(&self, torrent_ids: &[String]) -> anyhow::Result<()> {
        let id_values: Vec<RencodeValue> =
            torrent_ids.iter().map(|id| RencodeValue::Str(id.clone())).collect();
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.queue_bottom")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await
            .context("core.queue_bottom RPC failed")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "torrents/tests.rs"]
mod tests;
