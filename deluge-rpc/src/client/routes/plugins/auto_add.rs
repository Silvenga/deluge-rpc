use crate::DelugeRpcError;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{AutoAddConfig, WatchDirId, WatchDirOptions};
use crate::protocol::{DelugeRpcRequest, extract_single, extract_single_int};
use crate::{RencodeValue, to_rencode_value};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::BTreeMap;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait AutoAddRpc: Send + Sync {
    async fn set_options(
        &self,
        watch_dir_id: WatchDirId,
        options: &WatchDirOptions,
    ) -> Result<(), DelugeRpcError>;
    async fn enable_watch_dir(&self, watch_dir_id: WatchDirId) -> Result<(), DelugeRpcError>;
    async fn disable_watch_dir(&self, watch_dir_id: WatchDirId) -> Result<(), DelugeRpcError>;
    async fn set_config(&self, config: &AutoAddConfig) -> Result<(), DelugeRpcError>;
    async fn get_config(&self) -> Result<AutoAddConfig, DelugeRpcError>;
    async fn get_watch_dirs(&self) -> Result<BTreeMap<String, WatchDirOptions>, DelugeRpcError>;
    async fn add(&self, options: Option<WatchDirOptions>) -> Result<WatchDirId, DelugeRpcError>;
    async fn remove(&self, watch_dir_id: WatchDirId) -> Result<(), DelugeRpcError>;
    async fn is_admin_level(&self) -> Result<bool, DelugeRpcError>;
    async fn get_auth_user(&self) -> Result<String, DelugeRpcError>;
}

pub struct AutoAddClient {
    dispatcher: DelugeClientDispatcher,
}

impl AutoAddClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for AutoAddClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl AutoAddRpc for AutoAddClient {
    async fn set_options(
        &self,
        watch_dir_id: WatchDirId,
        options: &WatchDirOptions,
    ) -> Result<(), DelugeRpcError> {
        let options_value = to_rencode_value(options)?;
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("autoadd.set_options")
                    .with_args(vec![RencodeValue::Int(watch_dir_id), options_value]),
            )
            .await?;
        Ok(())
    }

    async fn enable_watch_dir(&self, watchdir_id: WatchDirId) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("autoadd.enable_watchdir")
                    .with_args(vec![RencodeValue::Int(watchdir_id)]),
            )
            .await?;
        Ok(())
    }

    async fn disable_watch_dir(&self, watchdir_id: WatchDirId) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("autoadd.disable_watchdir")
                    .with_args(vec![RencodeValue::Int(watchdir_id)]),
            )
            .await?;
        Ok(())
    }

    async fn set_config(&self, config: &AutoAddConfig) -> Result<(), DelugeRpcError> {
        let config_value = to_rencode_value(config)?;
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("autoadd.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> Result<AutoAddConfig, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("autoadd.get_config"))
            .await?;
        let value = extract_single(&result)?;
        Ok(AutoAddConfig::deserialize(&value)?)
    }

    async fn get_watch_dirs(&self) -> Result<BTreeMap<String, WatchDirOptions>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("autoadd.get_watchdirs"))
            .await?;
        let value = extract_single(&result)?;
        Ok(BTreeMap::<String, WatchDirOptions>::deserialize(&value)?)
    }

    async fn add(&self, options: Option<WatchDirOptions>) -> Result<WatchDirId, DelugeRpcError> {
        let args = match options {
            Some(opts) => vec![to_rencode_value(&opts)?],
            None => vec![RencodeValue::None],
        };
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("autoadd.add").with_args(args))
            .await?;
        let id = extract_single_int(&result, "autoadd.add")?;
        Ok(id)
    }

    async fn remove(&self, watchdir_id: WatchDirId) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("autoadd.remove")
                    .with_args(vec![RencodeValue::Int(watchdir_id)]),
            )
            .await?;
        Ok(())
    }

    async fn is_admin_level(&self) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("autoadd.is_admin_level"))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "autoadd.is_admin_level".into(),
                value: other,
            }),
        }
    }

    async fn get_auth_user(&self) -> Result<String, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("autoadd.get_auth_user"))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "autoadd.get_auth_user".into(),
                value: other,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    fn make_dict(entries: Vec<(&str, RencodeValue)>) -> RencodeValue {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(RencodeValue::Str(k.into()), v);
        }
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_autoadd_get_config_response_then_deserializes() {
        let mut watchdirs = BTreeMap::new();
        watchdirs.insert(
            RencodeValue::Str("1".into()),
            make_dict(vec![
                ("enabled", RencodeValue::Bool(true)),
                ("path", RencodeValue::Str("/watch".into())),
                ("append_extension", RencodeValue::Str(".added".into())),
                ("copy_torrent", RencodeValue::Bool(false)),
                ("delete_copy_torrent_toggle", RencodeValue::Bool(false)),
                ("abspath", RencodeValue::Bool(false)),
                ("download_location", RencodeValue::Str("".into())),
                ("max_download_speed", RencodeValue::Int(-1)),
                ("max_upload_speed", RencodeValue::Int(-1)),
                ("max_connections", RencodeValue::Int(-1)),
                ("max_upload_slots", RencodeValue::Int(-1)),
                ("prioritize_first_last", RencodeValue::Bool(false)),
                ("auto_managed", RencodeValue::Bool(true)),
                ("stop_at_ratio", RencodeValue::Bool(false)),
                ("stop_ratio", RencodeValue::Float(2.0)),
                ("remove_at_ratio", RencodeValue::Bool(false)),
                ("move_completed", RencodeValue::Bool(false)),
                ("move_completed_path", RencodeValue::Str("".into())),
                ("label", RencodeValue::Str("".into())),
                ("add_paused", RencodeValue::Bool(false)),
                ("queue_to_top", RencodeValue::Bool(false)),
                ("owner", RencodeValue::Str("admin".into())),
                ("seed_mode", RencodeValue::Bool(false)),
                ("max_download_speed_toggle", RencodeValue::Bool(false)),
                ("max_upload_speed_toggle", RencodeValue::Bool(false)),
                ("max_connections_toggle", RencodeValue::Bool(false)),
                ("max_upload_slots_toggle", RencodeValue::Bool(false)),
                ("download_location_toggle", RencodeValue::Bool(false)),
                ("move_completed_toggle", RencodeValue::Bool(false)),
                ("move_completed_path_toggle", RencodeValue::Bool(false)),
                ("add_paused_toggle", RencodeValue::Bool(false)),
                ("queue_to_top_toggle", RencodeValue::Bool(false)),
                ("auto_managed_toggle", RencodeValue::Bool(false)),
                ("stop_at_ratio_toggle", RencodeValue::Bool(false)),
                ("stop_ratio_toggle", RencodeValue::Bool(false)),
                ("remove_at_ratio_toggle", RencodeValue::Bool(false)),
                ("prioritize_first_last_toggle", RencodeValue::Bool(false)),
                ("seed_mode_toggle", RencodeValue::Bool(false)),
                ("label_toggle", RencodeValue::Bool(false)),
            ]),
        );

        let response = make_dict(vec![
            ("watchdirs", RencodeValue::Dict(watchdirs)),
            ("next_id", RencodeValue::Int(2)),
        ]);

        let value = extract_single(&response).expect("extract");
        let config: AutoAddConfig = AutoAddConfig::deserialize(&value).expect("deserialize");

        assert_eq!(config.next_id, 2);
        assert_eq!(config.watch_dirs.len(), 1);
    }

    #[test]
    fn when_autoadd_is_admin_level_response_then_deserializes_bool() {
        let response = RencodeValue::Bool(true);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }
}
