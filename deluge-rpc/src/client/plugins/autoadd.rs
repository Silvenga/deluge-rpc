use crate::client::RpcCaller;
use crate::models::plugins::{AutoAddConfig, WatchdirId, WatchdirOptions};
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use crate::protocol::extract_single_int;
use crate::rencode::{RencodeValue, to_rencode_value};
use crate::shared::Shared;
use crate::transport::DelugeWriter;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Deserialize;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait AutoaddRpc: Send + Sync {
    async fn set_options(&self, watchdir_id: WatchdirId, options: &WatchdirOptions) -> anyhow::Result<()>;
    async fn enable_watchdir(&self, watchdir_id: WatchdirId) -> anyhow::Result<()>;
    async fn disable_watchdir(&self, watchdir_id: WatchdirId) -> anyhow::Result<()>;
    async fn set_config(&self, config: &AutoAddConfig) -> anyhow::Result<()>;
    async fn get_config(&self) -> anyhow::Result<AutoAddConfig>;
    async fn get_watchdirs(&self) -> anyhow::Result<BTreeMap<String, WatchdirOptions>>;
    async fn add(&self, options: Option<WatchdirOptions>) -> anyhow::Result<WatchdirId>;
    async fn remove(&self, watchdir_id: WatchdirId) -> anyhow::Result<()>;
    async fn is_admin_level(&self) -> anyhow::Result<bool>;
    async fn get_auth_user(&self) -> anyhow::Result<String>;
}

pub struct AutoaddClient {
    caller: RpcCaller,
}

impl AutoaddClient {
    pub(crate) fn new(shared: Arc<Shared>, writer: Arc<Mutex<DelugeWriter>>) -> Self {
        Self {
            caller: RpcCaller::new(shared, writer),
        }
    }
}

impl Clone for AutoaddClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl AutoaddRpc for AutoaddClient {
    async fn set_options(&self, watchdir_id: WatchdirId, options: &WatchdirOptions) -> anyhow::Result<()> {
        let options_value = to_rencode_value(options).context("serializing watchdir options")?;
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("autoadd.set_options").with_args(vec![
                    RencodeValue::Int(watchdir_id),
                    options_value,
                ]),
            )
            .await?;
        Ok(())
    }

    async fn enable_watchdir(&self, watchdir_id: WatchdirId) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("autoadd.enable_watchdir").with_args(vec![
                    RencodeValue::Int(watchdir_id),
                ]),
            )
            .await?;
        Ok(())
    }

    async fn disable_watchdir(&self, watchdir_id: WatchdirId) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("autoadd.disable_watchdir").with_args(vec![
                    RencodeValue::Int(watchdir_id),
                ]),
            )
            .await?;
        Ok(())
    }

    async fn set_config(&self, config: &AutoAddConfig) -> anyhow::Result<()> {
        let config_value = to_rencode_value(config).context("serializing autoadd config")?;
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("autoadd.set_config").with_args(vec![config_value]),
            )
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> anyhow::Result<AutoAddConfig> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("autoadd.get_config"))
            .await
            .context("autoadd.get_config RPC failed")?;
        let value = extract_single(&result, "autoadd.get_config")?;
        AutoAddConfig::deserialize(&value).context("deserializing autoadd config")
    }

    async fn get_watchdirs(&self) -> anyhow::Result<BTreeMap<String, WatchdirOptions>> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("autoadd.get_watchdirs"))
            .await
            .context("autoadd.get_watchdirs RPC failed")?;
        let value = extract_single(&result, "autoadd.get_watchdirs")?;
        BTreeMap::<String, WatchdirOptions>::deserialize(&value)
            .context("deserializing watchdirs")
    }

    async fn add(&self, options: Option<WatchdirOptions>) -> anyhow::Result<WatchdirId> {
        let args = match options {
            Some(opts) => vec![to_rencode_value(&opts).context("serializing watchdir options")?],
            None => vec![RencodeValue::None],
        };
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("autoadd.add").with_args(args))
            .await
            .context("autoadd.add RPC failed")?;
        let id = extract_single_int(&result, "autoadd.add")?;
        Ok(id)
    }

    async fn remove(&self, watchdir_id: WatchdirId) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("autoadd.remove").with_args(vec![
                    RencodeValue::Int(watchdir_id),
                ]),
            )
            .await?;
        Ok(())
    }

    async fn is_admin_level(&self) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("autoadd.is_admin_level"))
            .await
            .context("autoadd.is_admin_level RPC failed")?;
        let value = extract_single(&result, "autoadd.is_admin_level")?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!("autoadd.is_admin_level returned non-bool: {other:?}")),
        }
    }

    async fn get_auth_user(&self) -> anyhow::Result<String> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("autoadd.get_auth_user"))
            .await
            .context("autoadd.get_auth_user RPC failed")?;
        let value = extract_single(&result, "autoadd.get_auth_user")?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(anyhow!("autoadd.get_auth_user returned non-str: {other:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;
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

        let response = RencodeValue::List(vec![make_dict(vec![
            ("watchdirs", RencodeValue::Dict(watchdirs)),
            ("next_id", RencodeValue::Int(2)),
        ])]);

        let value = extract_single(&response, "autoadd.get_config").expect("extract");
        let config: AutoAddConfig = AutoAddConfig::deserialize(&value).expect("deserialize");

        assert_eq!(config.next_id, 2);
        assert_eq!(config.watchdirs.len(), 1);
    }

    #[test]
    fn when_autoadd_is_admin_level_response_then_deserializes_bool() {
        let response = RencodeValue::List(vec![RencodeValue::Bool(true)]);
        let value = extract_single(&response, "autoadd.is_admin_level").expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }
}
