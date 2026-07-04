use crate::client::RpcCaller;
use crate::models::plugins::{SchedulerConfig, SchedulerState};
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use crate::rencode::to_rencode_value;
use crate::shared::Shared;
use crate::transport::DelugeWriter;
use anyhow::Context;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Deserialize;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait SchedulerRpc: Send + Sync {
    async fn set_config(&self, config: &SchedulerConfig) -> anyhow::Result<()>;
    async fn get_config(&self) -> anyhow::Result<SchedulerConfig>;
    async fn get_state(&self) -> anyhow::Result<SchedulerState>;
}

pub struct SchedulerClient {
    caller: RpcCaller,
}

impl SchedulerClient {
    pub(crate) fn new(shared: Arc<Shared>, writer: Arc<Mutex<DelugeWriter>>) -> Self {
        Self {
            caller: RpcCaller::new(shared, writer),
        }
    }
}

impl Clone for SchedulerClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl SchedulerRpc for SchedulerClient {
    async fn set_config(&self, config: &SchedulerConfig) -> anyhow::Result<()> {
        let config_value = to_rencode_value(config).context("serializing scheduler config")?;
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("scheduler.set_config").with_args(vec![config_value]),
            )
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> anyhow::Result<SchedulerConfig> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("scheduler.get_config"))
            .await
            .context("scheduler.get_config RPC failed")?;
        let value = extract_single(&result, "scheduler.get_config")?;
        SchedulerConfig::deserialize(&value).context("deserializing scheduler config")
    }

    async fn get_state(&self) -> anyhow::Result<SchedulerState> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("scheduler.get_state"))
            .await
            .context("scheduler.get_state RPC failed")?;
        let value = extract_single(&result, "scheduler.get_state")?;
        SchedulerState::deserialize(&value).context("deserializing scheduler state")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_scheduler_get_state_response_then_deserializes() {
        let response = RencodeValue::List(vec![RencodeValue::Str("Green".into())]);

        let value = extract_single(&response, "scheduler.get_state").expect("extract");
        let state: SchedulerState = SchedulerState::deserialize(&value).expect("deserialize");

        assert_eq!(state, SchedulerState::Green);
    }
}
