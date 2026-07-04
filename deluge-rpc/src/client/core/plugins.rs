use crate::client::RpcCaller;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use crate::rencode::RencodeValue;
use crate::shared::Shared;
use crate::transport::DelugeWriter;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait CorePluginRpc: Send + Sync {
    async fn get_available_plugins(&self) -> anyhow::Result<Vec<String>>;
    async fn get_enabled_plugins(&self) -> anyhow::Result<Vec<String>>;
    async fn enable_plugin(&self, plugin: &str) -> anyhow::Result<bool>;
    async fn disable_plugin(&self, plugin: &str) -> anyhow::Result<bool>;
    async fn upload_plugin(&self, filename: &str, filedump: &str) -> anyhow::Result<()>;
    async fn rescan_plugins(&self) -> anyhow::Result<()>;
}

pub struct CorePluginClient {
    caller: RpcCaller,
}

impl CorePluginClient {
    #[expect(dead_code, reason = "used in task 11 when connection code is updated")]
    pub(crate) fn new(shared: Arc<Shared>, writer: Arc<Mutex<DelugeWriter>>) -> Self {
        Self {
            caller: RpcCaller::new(shared, writer),
        }
    }
}

impl Clone for CorePluginClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl CorePluginRpc for CorePluginClient {
    async fn get_available_plugins(&self) -> anyhow::Result<Vec<String>> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_available_plugins"))
            .await
            .context("core.get_available_plugins RPC failed")?;
        let value = extract_single(&result, "core.get_available_plugins")?;
        match value {
            RencodeValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        RencodeValue::Str(s) => out.push(s),
                        other => {
                            return Err(anyhow!(
                                "core.get_available_plugins returned non-str element: {other:?}"
                            ))
                        }
                    }
                }
                Ok(out)
            }
            other => Err(anyhow!(
                "core.get_available_plugins returned non-list value: {other:?}"
            )),
        }
    }

    async fn get_enabled_plugins(&self) -> anyhow::Result<Vec<String>> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_enabled_plugins"))
            .await
            .context("core.get_enabled_plugins RPC failed")?;
        let value = extract_single(&result, "core.get_enabled_plugins")?;
        match value {
            RencodeValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        RencodeValue::Str(s) => out.push(s),
                        other => {
                            return Err(anyhow!(
                                "core.get_enabled_plugins returned non-str element: {other:?}"
                            ))
                        }
                    }
                }
                Ok(out)
            }
            other => Err(anyhow!(
                "core.get_enabled_plugins returned non-list value: {other:?}"
            )),
        }
    }

    async fn enable_plugin(&self, plugin: &str) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.enable_plugin")
                    .with_args(vec![RencodeValue::Str(plugin.to_owned())]),
            )
            .await
            .context("core.enable_plugin RPC failed")?;
        let value = extract_single(&result, "core.enable_plugin")?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "core.enable_plugin returned non-bool value: {other:?}"
            )),
        }
    }

    async fn disable_plugin(&self, plugin: &str) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.disable_plugin")
                    .with_args(vec![RencodeValue::Str(plugin.to_owned())]),
            )
            .await
            .context("core.disable_plugin RPC failed")?;
        let value = extract_single(&result, "core.disable_plugin")?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "core.disable_plugin returned non-bool value: {other:?}"
            )),
        }
    }

    async fn upload_plugin(&self, filename: &str, filedump: &str) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.upload_plugin").with_args(vec![
                    RencodeValue::Str(filename.to_owned()),
                    RencodeValue::Str(filedump.to_owned()),
                ]),
            )
            .await
            .context("core.upload_plugin RPC failed")?;
        Ok(())
    }

    async fn rescan_plugins(&self) -> anyhow::Result<()> {
        self.caller
            .rpc_call(DelugeRpcRequest::new("core.rescan_plugins"))
            .await
            .context("core.rescan_plugins RPC failed")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;

    #[test]
    fn when_core_get_available_plugins_then_vec_string() {
        let response = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Str("Label".into()),
            RencodeValue::Str("Blocklist".into()),
        ])]);
        let value = extract_single(&response, "core.get_available_plugins").expect("extract");
        match value {
            RencodeValue::List(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], RencodeValue::Str("Label".into()));
                assert_eq!(items[1], RencodeValue::Str("Blocklist".into()));
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn when_core_enable_plugin_then_bool() {
        let response = RencodeValue::List(vec![RencodeValue::Bool(true)]);
        let value = extract_single(&response, "core.enable_plugin").expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }
}
