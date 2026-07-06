use crate::client::dispatcher::DelugeClientDispatcher;
use crate::protocol::{extract_single, DelugeRpcRequest};
use crate::RencodeValue;
use anyhow::{anyhow, Context};
use async_trait::async_trait;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait ToggleRpc: Send + Sync {
    async fn get_status(&self) -> anyhow::Result<bool>;
    async fn toggle(&self) -> anyhow::Result<bool>;
}

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
    async fn get_status(&self) -> anyhow::Result<bool> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("toggle.get_status"))
            .await
            .context("toggle.get_status RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!("toggle.get_status returned non-bool: {other:?}")),
        }
    }

    async fn toggle(&self) -> anyhow::Result<bool> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("toggle.toggle"))
            .await
            .context("toggle.toggle RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!("toggle.toggle returned non-bool: {other:?}")),
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
