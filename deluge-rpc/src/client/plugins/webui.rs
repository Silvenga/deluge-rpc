use crate::client::RpcCaller;
use crate::models::plugins::WebUiConfig;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use crate::rencode::{RencodeValue, to_rencode_value};
use crate::shared::Shared;
use crate::transport::DelugeWriter;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Deserialize;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait WebuiRpc: Send + Sync {
    async fn got_deluge_web(&self) -> anyhow::Result<bool>;
    async fn set_config(&self, config: &WebUiConfig) -> anyhow::Result<()>;
    async fn get_config(&self) -> anyhow::Result<WebUiConfig>;
}

pub struct WebuiClient {
    caller: RpcCaller,
}

impl WebuiClient {
    pub(crate) fn new(shared: Arc<Shared>, writer: Arc<Mutex<DelugeWriter>>) -> Self {
        Self {
            caller: RpcCaller::new(shared, writer),
        }
    }
}

impl Clone for WebuiClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl WebuiRpc for WebuiClient {
    async fn got_deluge_web(&self) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("webui.got_deluge_web"))
            .await
            .context("webui.got_deluge_web RPC failed")?;
        let value = extract_single(&result, "webui.got_deluge_web")?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!("webui.got_deluge_web returned non-bool: {other:?}")),
        }
    }

    async fn set_config(&self, config: &WebUiConfig) -> anyhow::Result<()> {
        let config_value = to_rencode_value(config).context("serializing webui config")?;
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("webui.set_config").with_args(vec![config_value]),
            )
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> anyhow::Result<WebUiConfig> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("webui.get_config"))
            .await
            .context("webui.get_config RPC failed")?;
        let value = extract_single(&result, "webui.get_config")?;
        WebUiConfig::deserialize(&value).context("deserializing webui config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;

    #[test]
    fn when_webui_got_deluge_web_response_then_deserializes_bool() {
        let response = RencodeValue::List(vec![RencodeValue::Bool(true)]);

        let value = extract_single(&response, "webui.got_deluge_web").expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }
}
