use crate::client::dispatcher::DelugeClientDispatcher;
use crate::protocol::extract_single;
use crate::protocol::DelugeRpcRequest;
use crate::RencodeValue;
use anyhow::{anyhow, Context};
use async_trait::async_trait;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait CorePluginRpc: Send + Sync {
    async fn get_available_plugins(&self) -> anyhow::Result<Vec<String>>;
    async fn get_enabled_plugins(&self) -> anyhow::Result<Vec<String>>;
    async fn enable_plugin(&self, plugin: &str) -> anyhow::Result<bool>;
    async fn disable_plugin(&self, plugin: &str) -> anyhow::Result<bool>;
    async fn upload_plugin(&self, filename: &str, file_dump: &str) -> anyhow::Result<()>;
    async fn rescan_plugins(&self) -> anyhow::Result<()>;
}

pub struct CorePluginClient {
    dispatcher: DelugeClientDispatcher,
}

impl CorePluginClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for CorePluginClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl CorePluginRpc for CorePluginClient {
    async fn get_available_plugins(&self) -> anyhow::Result<Vec<String>> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.get_available_plugins"))
            .await
            .context("core.get_available_plugins RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        RencodeValue::Str(s) => out.push(s),
                        other => {
                            return Err(anyhow!(
                                "core.get_available_plugins returned non-str element: {other:?}"
                            ));
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
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.get_enabled_plugins"))
            .await
            .context("core.get_enabled_plugins RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        RencodeValue::Str(s) => out.push(s),
                        other => {
                            return Err(anyhow!(
                                "core.get_enabled_plugins returned non-str element: {other:?}"
                            ));
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
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.enable_plugin")
                    .with_args(vec![RencodeValue::Str(plugin.to_owned())]),
            )
            .await
            .context("core.enable_plugin RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "core.enable_plugin returned non-bool value: {other:?}"
            )),
        }
    }

    async fn disable_plugin(&self, plugin: &str) -> anyhow::Result<bool> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.disable_plugin")
                    .with_args(vec![RencodeValue::Str(plugin.to_owned())]),
            )
            .await
            .context("core.disable_plugin RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "core.disable_plugin returned non-bool value: {other:?}"
            )),
        }
    }

    async fn upload_plugin(&self, filename: &str, filedump: &str) -> anyhow::Result<()> {
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("core.upload_plugin").with_args(vec![
                RencodeValue::Str(filename.to_owned()),
                RencodeValue::Str(filedump.to_owned()),
            ]))
            .await
            .context("core.upload_plugin RPC failed")?;
        Ok(())
    }

    async fn rescan_plugins(&self) -> anyhow::Result<()> {
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("core.rescan_plugins"))
            .await
            .context("core.rescan_plugins RPC failed")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;

    #[test]
    fn when_core_get_available_plugins_then_vec_string() {
        let response = RencodeValue::List(vec![
            RencodeValue::Str("Label".into()),
            RencodeValue::Str("Blocklist".into()),
        ]);
        let value = extract_single(&response).expect("extract");
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
        let response = RencodeValue::Bool(true);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }
}
