use crate::DelugeRpcError;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::WebUiConfig;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use crate::{RencodeValue, to_rencode_value};
use async_trait::async_trait;
use serde::Deserialize;

/// RPC methods for the webui.* namespace.
#[async_trait]
pub trait WebUiRpc: Send + Sync {
    /// Returns `true` if the `deluge-web` module is installed and importable.
    async fn got_deluge_web(&self) -> Result<bool, DelugeRpcError>;
    /// Sets the plugin config.
    async fn set_config(&self, config: &WebUiConfig) -> Result<(), DelugeRpcError>;
    /// Returns the plugin config.
    async fn get_config(&self) -> Result<WebUiConfig, DelugeRpcError>;
}

/// Client for webui.* RPC methods.
pub struct WebUiClient {
    dispatcher: DelugeClientDispatcher,
}

impl WebUiClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for WebUiClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl WebUiRpc for WebUiClient {
    async fn got_deluge_web(&self) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("webui.got_deluge_web"))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "webui.got_deluge_web".into(),
                value: other,
            }),
        }
    }

    async fn set_config(&self, config: &WebUiConfig) -> Result<(), DelugeRpcError> {
        let config_value = to_rencode_value(config)?;
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("webui.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> Result<WebUiConfig, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("webui.get_config"))
            .await?;
        let value = extract_single(&result)?;
        Ok(WebUiConfig::deserialize(&value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;

    #[test]
    fn when_webui_got_deluge_web_response_then_deserializes_bool() {
        let response = RencodeValue::Bool(true);

        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }
}
