use crate::DelugeRpcError;
use crate::RencodeValue;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;

/// RPC methods for the `core.*` plugin namespace.
pub trait CorePluginRpc: Send + Sync {
    /// Returns names of all plugins available on the daemon.
    async fn get_available_plugins(&self) -> Result<Vec<String>, DelugeRpcError>;
    /// Returns names of currently enabled plugins.
    async fn get_enabled_plugins(&self) -> Result<Vec<String>, DelugeRpcError>;
    /// Enables a plugin. Returns `true` on success or if already enabled.
    async fn enable_plugin(&self, plugin: &str) -> Result<bool, DelugeRpcError>;
    /// Disables a plugin. Returns `true` on success or if already disabled.
    async fn disable_plugin(&self, plugin: &str) -> Result<bool, DelugeRpcError>;
    /// Uploads and installs a new plugin from base64-encoded data.
    async fn upload_plugin(&self, filename: &str, file_dump: &str) -> Result<(), DelugeRpcError>;
    /// Rescans the plugin folders for newly installed plugins.
    async fn rescan_plugins(&self) -> Result<(), DelugeRpcError>;
}

/// Client for `core.*` plugin RPC methods.
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

impl CorePluginRpc for CorePluginClient {
    async fn get_available_plugins(&self) -> Result<Vec<String>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.get_available_plugins"))
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
                                method: "core.get_available_plugins returned non-str element"
                                    .into(),
                                value: other,
                            });
                        }
                    }
                }
                Ok(out)
            }
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.get_available_plugins".into(),
                value: other,
            }),
        }
    }

    async fn get_enabled_plugins(&self) -> Result<Vec<String>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.get_enabled_plugins"))
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
                                method: "core.get_enabled_plugins returned non-str element".into(),
                                value: other,
                            });
                        }
                    }
                }
                Ok(out)
            }
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.get_enabled_plugins".into(),
                value: other,
            }),
        }
    }

    async fn enable_plugin(&self, plugin: &str) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.enable_plugin")
                    .with_args(vec![RencodeValue::Str(plugin.to_owned())]),
            )
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.enable_plugin".into(),
                value: other,
            }),
        }
    }

    async fn disable_plugin(&self, plugin: &str) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.disable_plugin")
                    .with_args(vec![RencodeValue::Str(plugin.to_owned())]),
            )
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.disable_plugin".into(),
                value: other,
            }),
        }
    }

    async fn upload_plugin(&self, filename: &str, filedump: &str) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("core.upload_plugin").with_args(vec![
                RencodeValue::Str(filename.to_owned()),
                RencodeValue::Str(filedump.to_owned()),
            ]))
            .await?;
        Ok(())
    }

    async fn rescan_plugins(&self) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("core.rescan_plugins"))
            .await?;
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
