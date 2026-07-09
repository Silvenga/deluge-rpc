use crate::DelugeRpcError;
use crate::RencodeValue;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::protocol::{DelugeRpcRequest, extract_single};
use async_trait::async_trait;

/// RPC methods for the `toggle.*` namespace.
#[async_trait]
pub trait ToggleRpc: Send + Sync {
    /// Returns `true` if the session is paused.
    async fn get_status(&self) -> Result<bool, DelugeRpcError>;
    /// Toggles the session between paused and running. Returns the new paused state.
    async fn toggle(&self) -> Result<bool, DelugeRpcError>;
}

/// Client for `toggle.*` RPC methods.
pub struct ToggleClient {
    dispatcher: DelugeClientDispatcher,
}

impl ToggleClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for ToggleClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl ToggleRpc for ToggleClient {
    async fn get_status(&self) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("toggle.get_status"))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "toggle.get_status".into(),
                value: other,
            }),
        }
    }

    async fn toggle(&self) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("toggle.toggle"))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "toggle.toggle".into(),
                value: other,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;

    #[test]
    fn when_toggle_get_status_response_then_deserializes_bool() {
        let response = RencodeValue::Bool(false);

        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(!b),
            other => panic!("expected bool, got {other:?}"),
        }
    }
}
