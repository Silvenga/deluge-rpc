use crate::DelugeRpcError;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{
    AddTorrentFileResult, AddTorrentFilesResult, AddTorrentOptions, FilterDict, FilterTree,
    GetMagnetUriResult, PrefetchMagnetResult, RemoveTorrentsResult, SetTorrentOptions,
    TorrentEntry, TorrentStatus, TrackerInfo,
};
use crate::protocol::{DelugeRpcRequest, extract_single, extract_single_dict, extract_single_int};
use crate::{RencodeValue, to_rencode_value};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::BTreeMap;

#[async_trait]
pub trait CoreTorrentRpc: Send + Sync {
    async fn add_torrent_file(
        &self,
        filename: &str,
        filedump: &str,
        options: &AddTorrentOptions,
    ) -> Result<AddTorrentFileResult, DelugeRpcError>;
    async fn add_torrent_file_async(
        &self,
        filename: &str,
        filedump: &str,
        options: &AddTorrentOptions,
        save_state: bool,
    ) -> Result<AddTorrentFileResult, DelugeRpcError>;
    async fn add_torrent_files(
        &self,
        torrent_files: &[(String, String, AddTorrentOptions)],
    ) -> Result<AddTorrentFilesResult, DelugeRpcError>;
    async fn add_torrent_url(
        &self,
        url: &str,
        options: &AddTorrentOptions,
        headers: Option<BTreeMap<String, String>>,
    ) -> Result<AddTorrentFileResult, DelugeRpcError>;
    async fn add_torrent_magnet(
        &self,
        uri: &str,
        options: &AddTorrentOptions,
    ) -> Result<String, DelugeRpcError>;
    async fn prefetch_magnet_metadata(
        &self,
        magnet: &str,
        timeout_secs: Option<i64>,
    ) -> Result<PrefetchMagnetResult, DelugeRpcError>;
    async fn remove_torrent(
        &self,
        torrent_id: &str,
        remove_data: bool,
    ) -> Result<bool, DelugeRpcError>;
    async fn remove_torrents(
        &self,
        torrent_ids: &[String],
        remove_data: bool,
    ) -> Result<RemoveTorrentsResult, DelugeRpcError>;
    async fn pause_torrent(&self, torrent_id: &str) -> Result<(), DelugeRpcError>;
    async fn pause_torrents(&self, torrent_ids: Option<Vec<String>>) -> Result<(), DelugeRpcError>;
    async fn resume_torrent(&self, torrent_id: &str) -> Result<(), DelugeRpcError>;
    async fn resume_torrents(&self, torrent_ids: Option<Vec<String>>)
    -> Result<(), DelugeRpcError>;
    async fn force_reannounce(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError>;
    async fn force_recheck(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError>;
    async fn set_torrent_options(
        &self,
        torrent_ids: &[String],
        options: &SetTorrentOptions,
    ) -> Result<(), DelugeRpcError>;
    async fn connect_peer(
        &self,
        torrent_id: &str,
        ip: &str,
        port: i64,
    ) -> Result<(), DelugeRpcError>;
    async fn move_storage(&self, torrent_ids: &[String], dest: &str) -> Result<(), DelugeRpcError>;
    async fn set_ssl_torrent_cert(
        &self,
        torrent_id: &str,
        certificate: &str,
        private_key: &str,
        dh_params: &str,
        save_to_disk: bool,
    ) -> Result<(), DelugeRpcError>;
    async fn get_torrent_status(
        &self,
        torrent_id: &str,
        keys: &[String],
        diff: bool,
    ) -> Result<TorrentStatus, DelugeRpcError>;
    async fn get_torrents_status(
        &self,
        filter_dict: &FilterDict,
        keys: &[String],
        diff: bool,
    ) -> Result<Vec<TorrentEntry>, DelugeRpcError>;
    async fn get_filter_tree(
        &self,
        show_zero_hits: bool,
        hide_cat: Option<Vec<String>>,
    ) -> Result<FilterTree, DelugeRpcError>;
    async fn get_session_state(&self) -> Result<Vec<String>, DelugeRpcError>;
    async fn get_magnet_uri(&self, torrent_id: &str) -> Result<GetMagnetUriResult, DelugeRpcError>;
    async fn get_path_size(&self, path: &str) -> Result<i64, DelugeRpcError>;
    async fn set_torrent_trackers(
        &self,
        torrent_id: &str,
        trackers: &[TrackerInfo],
    ) -> Result<(), DelugeRpcError>;
    async fn rename_files(
        &self,
        torrent_id: &str,
        filenames: &[(i64, String)],
    ) -> Result<(), DelugeRpcError>;
    async fn rename_folder(
        &self,
        torrent_id: &str,
        folder: &str,
        new_folder: &str,
    ) -> Result<(), DelugeRpcError>;
    async fn queue_top(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError>;
    async fn queue_up(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError>;
    async fn queue_down(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError>;
    async fn queue_bottom(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError>;
}

pub struct CoreTorrentClient {
    dispatcher: DelugeClientDispatcher,
}

impl CoreTorrentClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for CoreTorrentClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl CoreTorrentRpc for CoreTorrentClient {
    async fn add_torrent_file(
        &self,
        filename: &str,
        filedump: &str,
        options: &AddTorrentOptions,
    ) -> Result<AddTorrentFileResult, DelugeRpcError> {
        let options_value = to_rencode_value(options)?;
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.add_torrent_file").with_args(vec![
                    RencodeValue::Str(filename.to_owned()),
                    RencodeValue::Str(filedump.to_owned()),
                    options_value,
                ]),
            )
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(Some(s)),
            RencodeValue::None => Ok(None),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.add_torrent_file returned unexpected value".into(),
                value: other,
            }),
        }
    }

    async fn add_torrent_file_async(
        &self,
        filename: &str,
        filedump: &str,
        options: &AddTorrentOptions,
        save_state: bool,
    ) -> Result<AddTorrentFileResult, DelugeRpcError> {
        let options_value = to_rencode_value(options)?;
        let mut kwargs = BTreeMap::new();
        kwargs.insert(
            RencodeValue::Str("save_state".into()),
            RencodeValue::Bool(save_state),
        );
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.add_torrent_file_async")
                    .with_args(vec![
                        RencodeValue::Str(filename.to_owned()),
                        RencodeValue::Str(filedump.to_owned()),
                        options_value,
                    ])
                    .with_kwargs(kwargs),
            )
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(Some(s)),
            RencodeValue::None => Ok(None),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.add_torrent_file_async returned unexpected value".into(),
                value: other,
            }),
        }
    }

    async fn add_torrent_files(
        &self,
        torrent_files: &[(String, String, AddTorrentOptions)],
    ) -> Result<AddTorrentFilesResult, DelugeRpcError> {
        let mut items = Vec::with_capacity(torrent_files.len());
        for (filename, filedump, options) in torrent_files {
            let options_value = to_rencode_value(options)?;
            items.push(RencodeValue::List(vec![
                RencodeValue::Str(filename.clone()),
                RencodeValue::Str(filedump.clone()),
                options_value,
            ]));
        }
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.add_torrent_files")
                    .with_args(vec![RencodeValue::List(items)]),
            )
            .await?;
        let value = extract_single(&result)?;
        Ok(AddTorrentFilesResult::deserialize(&value)?)
    }

    async fn add_torrent_url(
        &self,
        url: &str,
        options: &AddTorrentOptions,
        headers: Option<BTreeMap<String, String>>,
    ) -> Result<AddTorrentFileResult, DelugeRpcError> {
        let options_value = to_rencode_value(options)?;
        let mut kwargs = BTreeMap::new();
        if let Some(h) = headers {
            let headers_value = to_rencode_value(&h)?;
            kwargs.insert(RencodeValue::Str("headers".into()), headers_value);
        }
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.add_torrent_url")
                    .with_args(vec![RencodeValue::Str(url.to_owned()), options_value])
                    .with_kwargs(kwargs),
            )
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(Some(s)),
            RencodeValue::None => Ok(None),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.add_torrent_url returned unexpected value".into(),
                value: other,
            }),
        }
    }

    async fn add_torrent_magnet(
        &self,
        uri: &str,
        options: &AddTorrentOptions,
    ) -> Result<String, DelugeRpcError> {
        let options_value = to_rencode_value(options)?;
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.add_torrent_magnet")
                    .with_args(vec![RencodeValue::Str(uri.to_owned()), options_value]),
            )
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.add_torrent_magnet".into(),
                value: other,
            }),
        }
    }

    async fn prefetch_magnet_metadata(
        &self,
        magnet: &str,
        timeout_secs: Option<i64>,
    ) -> Result<PrefetchMagnetResult, DelugeRpcError> {
        let mut kwargs = BTreeMap::new();
        if let Some(t) = timeout_secs {
            kwargs.insert(RencodeValue::Str("timeout".into()), RencodeValue::Int(t));
        }
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.prefetch_magnet_metadata")
                    .with_args(vec![RencodeValue::Str(magnet.to_owned())])
                    .with_kwargs(kwargs),
            )
            .await?;
        let value = extract_single(&result)?;
        Ok(PrefetchMagnetResult::deserialize(&value)?)
    }

    async fn remove_torrent(
        &self,
        torrent_id: &str,
        remove_data: bool,
    ) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.remove_torrent").with_args(vec![
                RencodeValue::Str(torrent_id.to_owned()),
                RencodeValue::Bool(remove_data),
            ]))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.remove_torrent".into(),
                value: other,
            }),
        }
    }

    async fn remove_torrents(
        &self,
        torrent_ids: &[String],
        remove_data: bool,
    ) -> Result<RemoveTorrentsResult, DelugeRpcError> {
        let ids: Vec<RencodeValue> = torrent_ids
            .iter()
            .map(|id| RencodeValue::Str(id.clone()))
            .collect();
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.remove_torrents").with_args(vec![
                    RencodeValue::List(ids),
                    RencodeValue::Bool(remove_data),
                ]),
            )
            .await?;
        let value = extract_single(&result)?;
        Ok(RemoveTorrentsResult::deserialize(&value)?)
    }

    async fn pause_torrent(&self, torrent_id: &str) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.pause_torrent")
                    .with_args(vec![RencodeValue::Str(torrent_id.to_owned())]),
            )
            .await?;
        Ok(())
    }

    async fn pause_torrents(&self, torrent_ids: Option<Vec<String>>) -> Result<(), DelugeRpcError> {
        let args = match torrent_ids {
            Some(ids) => {
                let id_values: Vec<RencodeValue> = ids.into_iter().map(RencodeValue::Str).collect();
                vec![RencodeValue::List(id_values)]
            }
            None => vec![RencodeValue::None],
        };
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("core.pause_torrents").with_args(args))
            .await?;
        Ok(())
    }

    async fn resume_torrent(&self, torrent_id: &str) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.resume_torrent")
                    .with_args(vec![RencodeValue::Str(torrent_id.to_owned())]),
            )
            .await?;
        Ok(())
    }

    async fn resume_torrents(
        &self,
        torrent_ids: Option<Vec<String>>,
    ) -> Result<(), DelugeRpcError> {
        let args = match torrent_ids {
            Some(ids) => {
                let id_values: Vec<RencodeValue> = ids.into_iter().map(RencodeValue::Str).collect();
                vec![RencodeValue::List(id_values)]
            }
            None => vec![RencodeValue::None],
        };
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("core.resume_torrents").with_args(args))
            .await?;
        Ok(())
    }

    async fn force_reannounce(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError> {
        let id_values: Vec<RencodeValue> = torrent_ids
            .iter()
            .map(|id| RencodeValue::Str(id.clone()))
            .collect();
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.force_reannounce")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await?;
        Ok(())
    }

    async fn force_recheck(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError> {
        let id_values: Vec<RencodeValue> = torrent_ids
            .iter()
            .map(|id| RencodeValue::Str(id.clone()))
            .collect();
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.force_recheck")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await?;
        Ok(())
    }

    async fn set_torrent_options(
        &self,
        torrent_ids: &[String],
        options: &SetTorrentOptions,
    ) -> Result<(), DelugeRpcError> {
        let id_values: Vec<RencodeValue> = torrent_ids
            .iter()
            .map(|id| RencodeValue::Str(id.clone()))
            .collect();
        let options_value = to_rencode_value(options)?;
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.set_torrent_options")
                    .with_args(vec![RencodeValue::List(id_values), options_value]),
            )
            .await?;
        Ok(())
    }

    async fn connect_peer(
        &self,
        torrent_id: &str,
        ip: &str,
        port: i64,
    ) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("core.connect_peer").with_args(vec![
                RencodeValue::Str(torrent_id.to_owned()),
                RencodeValue::Str(ip.to_owned()),
                RencodeValue::Int(port),
            ]))
            .await?;
        Ok(())
    }

    async fn move_storage(&self, torrent_ids: &[String], dest: &str) -> Result<(), DelugeRpcError> {
        let id_values: Vec<RencodeValue> = torrent_ids
            .iter()
            .map(|id| RencodeValue::Str(id.clone()))
            .collect();
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("core.move_storage").with_args(vec![
                RencodeValue::List(id_values),
                RencodeValue::Str(dest.to_owned()),
            ]))
            .await?;
        Ok(())
    }

    async fn set_ssl_torrent_cert(
        &self,
        torrent_id: &str,
        certificate: &str,
        private_key: &str,
        dh_params: &str,
        save_to_disk: bool,
    ) -> Result<(), DelugeRpcError> {
        let mut kwargs = BTreeMap::new();
        kwargs.insert(
            RencodeValue::Str("save_to_disk".into()),
            RencodeValue::Bool(save_to_disk),
        );
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.set_ssl_torrent_cert")
                    .with_args(vec![
                        RencodeValue::Str(torrent_id.to_owned()),
                        RencodeValue::Str(certificate.to_owned()),
                        RencodeValue::Str(private_key.to_owned()),
                        RencodeValue::Str(dh_params.to_owned()),
                    ])
                    .with_kwargs(kwargs),
            )
            .await?;
        Ok(())
    }

    async fn get_torrent_status(
        &self,
        torrent_id: &str,
        keys: &[String],
        diff: bool,
    ) -> Result<TorrentStatus, DelugeRpcError> {
        let key_values: Vec<RencodeValue> =
            keys.iter().map(|k| RencodeValue::Str(k.clone())).collect();
        let mut kwargs = BTreeMap::new();
        kwargs.insert(RencodeValue::Str("diff".into()), RencodeValue::Bool(diff));
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.get_torrent_status")
                    .with_args(vec![
                        RencodeValue::Str(torrent_id.to_owned()),
                        RencodeValue::List(key_values),
                    ])
                    .with_kwargs(kwargs),
            )
            .await?;
        let value = extract_single(&result)?;
        Ok(TorrentStatus::deserialize(&value)?)
    }

    async fn get_torrents_status(
        &self,
        filter_dict: &FilterDict,
        keys: &[String],
        diff: bool,
    ) -> Result<Vec<TorrentEntry>, DelugeRpcError> {
        let filter_value = to_rencode_value(filter_dict)?;
        let key_values: Vec<RencodeValue> =
            keys.iter().map(|k| RencodeValue::Str(k.clone())).collect();
        let mut kwargs = BTreeMap::new();
        kwargs.insert(RencodeValue::Str("diff".into()), RencodeValue::Bool(diff));
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.get_torrents_status")
                    .with_args(vec![filter_value, RencodeValue::List(key_values)])
                    .with_kwargs(kwargs),
            )
            .await?;

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
            let mut entry = TorrentEntry::deserialize(fields).map_err(DelugeRpcError::from)?;
            entry.info_hash = info_hash;
            out.push(entry);
        }
        Ok(out)
    }

    async fn get_filter_tree(
        &self,
        show_zero_hits: bool,
        hide_cat: Option<Vec<String>>,
    ) -> Result<FilterTree, DelugeRpcError> {
        let mut kwargs = BTreeMap::new();
        kwargs.insert(
            RencodeValue::Str("show_zero_hits".into()),
            RencodeValue::Bool(show_zero_hits),
        );
        if let Some(cats) = hide_cat {
            let cat_values: Vec<RencodeValue> = cats.into_iter().map(RencodeValue::Str).collect();
            kwargs.insert(
                RencodeValue::Str("hide_cat".into()),
                RencodeValue::List(cat_values),
            );
        }
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.get_filter_tree").with_kwargs(kwargs))
            .await?;
        let value = extract_single(&result)?;
        Ok(FilterTree::deserialize(&value)?)
    }

    async fn get_session_state(&self) -> Result<Vec<String>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.get_session_state"))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        RencodeValue::Str(s) => out.push(s),
                        other => {
                            return Err(DelugeRpcError::UnexpectedResponseType {
                                method: "core.get_session_state returned non-str element".into(),
                                value: other,
                            });
                        }
                    }
                }
                Ok(out)
            }
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.get_session_state".into(),
                value: other,
            }),
        }
    }

    async fn get_magnet_uri(&self, torrent_id: &str) -> Result<GetMagnetUriResult, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.get_magnet_uri")
                    .with_args(vec![RencodeValue::Str(torrent_id.to_owned())]),
            )
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.get_magnet_uri".into(),
                value: other,
            }),
        }
    }

    async fn get_path_size(&self, path: &str) -> Result<i64, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.get_path_size")
                    .with_args(vec![RencodeValue::Str(path.to_owned())]),
            )
            .await?;
        Ok(extract_single_int(&result, "core.get_path_size")?)
    }

    async fn set_torrent_trackers(
        &self,
        torrent_id: &str,
        trackers: &[TrackerInfo],
    ) -> Result<(), DelugeRpcError> {
        let tracker_values = to_rencode_value(trackers)?;
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.set_torrent_trackers").with_args(vec![
                    RencodeValue::Str(torrent_id.to_owned()),
                    tracker_values,
                ]),
            )
            .await?;
        Ok(())
    }

    async fn rename_files(
        &self,
        torrent_id: &str,
        filenames: &[(i64, String)],
    ) -> Result<(), DelugeRpcError> {
        let file_values: Vec<RencodeValue> = filenames
            .iter()
            .map(|(idx, name)| {
                RencodeValue::List(vec![
                    RencodeValue::Int(*idx),
                    RencodeValue::Str(name.clone()),
                ])
            })
            .collect();
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("core.rename_files").with_args(vec![
                RencodeValue::Str(torrent_id.to_owned()),
                RencodeValue::List(file_values),
            ]))
            .await?;
        Ok(())
    }

    async fn rename_folder(
        &self,
        torrent_id: &str,
        folder: &str,
        new_folder: &str,
    ) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("core.rename_folder").with_args(vec![
                RencodeValue::Str(torrent_id.to_owned()),
                RencodeValue::Str(folder.to_owned()),
                RencodeValue::Str(new_folder.to_owned()),
            ]))
            .await?;
        Ok(())
    }

    async fn queue_top(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError> {
        let id_values: Vec<RencodeValue> = torrent_ids
            .iter()
            .map(|id| RencodeValue::Str(id.clone()))
            .collect();
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.queue_top")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await?;
        Ok(())
    }

    async fn queue_up(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError> {
        let id_values: Vec<RencodeValue> = torrent_ids
            .iter()
            .map(|id| RencodeValue::Str(id.clone()))
            .collect();
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.queue_up")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await?;
        Ok(())
    }

    async fn queue_down(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError> {
        let id_values: Vec<RencodeValue> = torrent_ids
            .iter()
            .map(|id| RencodeValue::Str(id.clone()))
            .collect();
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.queue_down")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await?;
        Ok(())
    }

    async fn queue_bottom(&self, torrent_ids: &[String]) -> Result<(), DelugeRpcError> {
        let id_values: Vec<RencodeValue> = torrent_ids
            .iter()
            .map(|id| RencodeValue::Str(id.clone()))
            .collect();
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.queue_bottom")
                    .with_args(vec![RencodeValue::List(id_values)]),
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use crate::models::{
        AddTorrentFileResult, AddTorrentFilesResult, FilterTree, PrefetchMagnetResult,
        RemoveTorrentsResult, TorrentEntry, TorrentStatus,
    };
    use crate::protocol::{extract_single, extract_single_dict, extract_single_int};
    use std::collections::BTreeMap;

    #[test]
    fn when_add_torrent_file_response_str_then_some() {
        let response = RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into());
        let value = extract_single(&response).expect("extract");
        let result: AddTorrentFileResult = match value {
            RencodeValue::Str(s) => Some(s),
            RencodeValue::None => None,
            _ => panic!("unexpected value: {value:?}"),
        };
        assert_eq!(
            result,
            Some("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into())
        );
    }

    #[test]
    fn when_add_torrent_file_response_none_then_none() {
        let response = RencodeValue::None;
        let value = extract_single(&response).expect("extract");
        let result: AddTorrentFileResult = match value {
            RencodeValue::Str(s) => Some(s),
            RencodeValue::None => None,
            _ => panic!("unexpected value: {value:?}"),
        };
        assert_eq!(result, None);
    }

    #[test]
    fn when_add_torrent_files_response_empty_then_all_succeeded() {
        let response = RencodeValue::List(vec![]);
        let value = extract_single(&response).expect("extract");
        let result: AddTorrentFilesResult =
            AddTorrentFilesResult::deserialize(&value).expect("deserialize");
        assert!(result.is_empty());
    }

    #[test]
    fn when_add_torrent_files_response_errors_then_deserialized() {
        let response = RencodeValue::List(vec![RencodeValue::Str("failed to add torrent".into())]);
        let value = extract_single(&response).expect("extract");
        let result: AddTorrentFilesResult =
            AddTorrentFilesResult::deserialize(&value).expect("deserialize");
        assert_eq!(result, vec!["failed to add torrent"]);
    }

    #[test]
    fn when_add_torrent_magnet_response_str_then_ok() {
        let response = RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into());
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Str(s) => assert_eq!(s, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111"),
            other => panic!("expected str, got {other:?}"),
        }
    }

    #[test]
    fn when_prefetch_magnet_metadata_response_tuple_then_deserialized() {
        let response = RencodeValue::List(vec![
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
            RencodeValue::Bytes(b"bencoded-data".to_vec()),
        ]);
        let value = extract_single(&response).expect("extract");
        let result: PrefetchMagnetResult =
            PrefetchMagnetResult::deserialize(&value).expect("deserialize");
        assert_eq!(
            result.torrent_id,
            "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111"
        );
        assert_eq!(result.metadata, b"bencoded-data");
    }

    #[test]
    fn when_remove_torrent_response_true_then_bool() {
        let response = RencodeValue::Bool(true);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[test]
    fn when_remove_torrent_response_false_then_bool() {
        let response = RencodeValue::Bool(false);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(!b),
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[test]
    fn when_remove_torrents_response_empty_then_all_succeeded() {
        let response = RencodeValue::List(vec![]);
        let value = extract_single(&response).expect("extract");
        let result: RemoveTorrentsResult =
            RemoveTorrentsResult::deserialize(&value).expect("deserialize");
        assert!(result.is_empty());
    }

    #[test]
    fn when_remove_torrents_response_errors_then_deserialized() {
        let response = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
            RencodeValue::Str("torrent not found".into()),
        ])]);
        let value = extract_single(&response).expect("extract");
        let result: RemoveTorrentsResult =
            RemoveTorrentsResult::deserialize(&value).expect("deserialize");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
        assert_eq!(result[0].1, "torrent not found");
    }

    #[test]
    fn when_get_torrent_status_response_dict_then_torrent_status() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("name".into()),
            RencodeValue::Str("test-torrent".into()),
        );
        map.insert(
            RencodeValue::Str("state".into()),
            RencodeValue::Str("Downloading".into()),
        );
        map.insert(
            RencodeValue::Str("progress".into()),
            RencodeValue::Float(50.0),
        );
        map.insert(
            RencodeValue::Str("hash".into()),
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
        );
        map.insert(RencodeValue::Str("ratio".into()), RencodeValue::Float(-1.0));
        map.insert(RencodeValue::Str("eta".into()), RencodeValue::Int(-1));
        map.insert(
            RencodeValue::Str("completed_time".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("last_seen_complete".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("time_since_download".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("time_since_transfer".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("time_since_upload".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("seeds_peers_ratio".into()),
            RencodeValue::Float(-1.0),
        );
        map.insert(
            RencodeValue::Str("max_connections".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_download_speed".into()),
            RencodeValue::Float(-1.0),
        );
        map.insert(
            RencodeValue::Str("max_upload_slots".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_upload_speed".into()),
            RencodeValue::Float(-1.0),
        );
        let response = RencodeValue::Dict(map);

        let value = extract_single(&response).expect("extract");
        let status: TorrentStatus = TorrentStatus::deserialize(&value).expect("deserialize");

        assert_eq!(status.name, "test-torrent");
        assert_eq!(status.state, "Downloading");
        assert!((status.progress - 50.0).abs() < f64::EPSILON);
        assert_eq!(status.ratio, None);
        assert_eq!(status.eta, None);
    }

    fn make_torrent_entry_dict(name: &str, hash: &str) -> BTreeMap<RencodeValue, RencodeValue> {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("name".into()),
            RencodeValue::Str(name.into()),
        );
        map.insert(
            RencodeValue::Str("state".into()),
            RencodeValue::Str("Seeding".into()),
        );
        map.insert(
            RencodeValue::Str("progress".into()),
            RencodeValue::Float(100.0),
        );
        map.insert(RencodeValue::Str("ratio".into()), RencodeValue::Float(2.5));
        map.insert(
            RencodeValue::Str("hash".into()),
            RencodeValue::Str(hash.into()),
        );
        map.insert(RencodeValue::Str("eta".into()), RencodeValue::Int(-1));
        map.insert(
            RencodeValue::Str("completed_time".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("last_seen_complete".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("time_since_download".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("time_since_transfer".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("time_since_upload".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("seeds_peers_ratio".into()),
            RencodeValue::Float(-1.0),
        );
        map.insert(
            RencodeValue::Str("max_connections".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_download_speed".into()),
            RencodeValue::Float(-1.0),
        );
        map.insert(
            RencodeValue::Str("max_upload_slots".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_upload_speed".into()),
            RencodeValue::Float(-1.0),
        );
        map
    }

    #[test]
    fn when_get_torrents_status_response_dict_then_vec_torrent_entry() {
        let mut result_dict = BTreeMap::new();
        result_dict.insert(
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
            RencodeValue::Dict(make_torrent_entry_dict(
                "torrent-a",
                "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111",
            )),
        );
        result_dict.insert(
            RencodeValue::Str("bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222".into()),
            RencodeValue::Dict(make_torrent_entry_dict(
                "torrent-b",
                "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222",
            )),
        );
        let response = RencodeValue::Dict(result_dict);

        let result_dict =
            extract_single_dict(&response, "core.get_torrents_status").expect("extract");

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
                .unwrap_or_else(|_| panic!("deserialize {info_hash}"));
            entry.info_hash = info_hash;
            out.push(entry);
        }

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].info_hash, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
        assert_eq!(out[0].status.name, "torrent-a");
        assert_eq!(out[1].info_hash, "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222");
        assert_eq!(out[1].status.name, "torrent-b");
    }

    #[test]
    fn when_get_torrents_status_response_empty_dict_then_empty_vec() {
        let response = RencodeValue::Dict(BTreeMap::new());

        let result_dict =
            extract_single_dict(&response, "core.get_torrents_status").expect("extract");
        assert!(result_dict.is_empty());
    }

    #[test]
    fn when_get_filter_tree_response_dict_then_filter_tree() {
        let state_entries = vec![
            RencodeValue::List(vec![RencodeValue::Str("All".into()), RencodeValue::Int(42)]),
            RencodeValue::List(vec![
                RencodeValue::Str("Seeding".into()),
                RencodeValue::Int(10),
            ]),
        ];

        let mut filter_dict = BTreeMap::new();
        filter_dict.insert(
            RencodeValue::Str("state".into()),
            RencodeValue::List(state_entries),
        );

        let response = RencodeValue::Dict(filter_dict);

        let value = extract_single(&response).expect("extract");
        let tree: FilterTree = FilterTree::deserialize(&value).expect("deserialize");

        let state = tree.get("state").expect("state key");
        assert_eq!(state.len(), 2);
        assert_eq!(state[0].value, "All");
        assert_eq!(state[0].count, 42);
        assert_eq!(state[1].value, "Seeding");
        assert_eq!(state[1].count, 10);
    }

    #[test]
    fn when_get_session_state_response_list_then_vec_string() {
        let response = RencodeValue::List(vec![
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
            RencodeValue::Str("bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222".into()),
        ]);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        RencodeValue::Str(s) => out.push(s),
                        other => panic!("expected str, got {other:?}"),
                    }
                }
                assert_eq!(out.len(), 2);
                assert_eq!(out[0], "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
                assert_eq!(out[1], "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222");
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn when_get_session_state_response_empty_list_then_empty_vec() {
        let response = RencodeValue::List(vec![]);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::List(items) => assert!(items.is_empty()),
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn when_get_magnet_uri_response_str_then_string() {
        let response = RencodeValue::Str(
            "magnet:?xt=urn:btih:aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into(),
        );
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Str(s) => {
                assert_eq!(
                    s,
                    "magnet:?xt=urn:btih:aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111"
                );
            }
            other => panic!("expected str, got {other:?}"),
        }
    }

    #[test]
    fn when_get_path_size_response_int_then_i64() {
        let response = RencodeValue::Int(1_073_741_824);
        let bytes = extract_single_int(&response, "core.get_path_size").expect("extract");
        assert_eq!(bytes, 1_073_741_824);
    }
}
