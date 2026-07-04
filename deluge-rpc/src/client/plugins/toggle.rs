use crate::client::RpcCaller;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use crate::rencode::RencodeValue;
use anyhow::{Context, anyhow};
use async_trait::async_trait;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait ToggleRpc: Send + Sync {
    async fn get_status(&self) -> anyhow::Result<bool>;
    async fn toggle(&self) -> anyhow::Result<bool>;
}

pub struct ToggleClient {
    caller: RpcCaller,
}

impl ToggleClient {
    pub(crate) fn new(caller: RpcCaller) -> Self {
        Self { caller }
    }
}

impl Clone for ToggleClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl ToggleRpc for ToggleClient {
    async fn get_status(&self) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("toggle.get_status"))
            .await
            .context("toggle.get_status RPC failed")?;
        let value = extract_single(&result, "toggle.get_status")?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!("toggle.get_status returned non-bool: {other:?}")),
        }
    }

    async fn toggle(&self) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("toggle.toggle"))
            .await
            .context("toggle.toggle RPC failed")?;
        let value = extract_single(&result, "toggle.toggle")?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!("toggle.toggle returned non-bool: {other:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;

    #[test]
    fn when_toggle_get_status_response_then_deserializes_bool() {
        let response = RencodeValue::List(vec![RencodeValue::Bool(false)]);

        let value = extract_single(&response, "toggle.get_status").expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(!b),
            other => panic!("expected bool, got {other:?}"),
        }
    }
}
