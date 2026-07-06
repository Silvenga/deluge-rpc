use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{SchedulerConfig, SchedulerState};
use crate::protocol::{extract_single, DelugeRpcRequest};
use crate::to_rencode_value;
use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait SchedulerRpc: Send + Sync {
    async fn set_config(&self, config: &SchedulerConfig) -> anyhow::Result<()>;
    async fn get_config(&self) -> anyhow::Result<SchedulerConfig>;
    async fn get_state(&self) -> anyhow::Result<SchedulerState>;
}

pub struct SchedulerClient {
    dispatcher: DelugeClientDispatcher,
}

impl SchedulerClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for SchedulerClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl SchedulerRpc for SchedulerClient {
    async fn set_config(&self, config: &SchedulerConfig) -> anyhow::Result<()> {
        let config_value = to_rencode_value(config).context("serializing scheduler config")?;
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("scheduler.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> anyhow::Result<SchedulerConfig> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("scheduler.get_config"))
            .await
            .context("scheduler.get_config RPC failed")?;
        let value = extract_single(&result)?;
        SchedulerConfig::deserialize(&value).context("deserializing scheduler config")
    }

    async fn get_state(&self) -> anyhow::Result<SchedulerState> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("scheduler.get_state"))
            .await
            .context("scheduler.get_state RPC failed")?;
        let value = extract_single(&result)?;
        SchedulerState::deserialize(&value).context("deserializing scheduler state")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_scheduler_get_state_response_then_deserializes() {
        let response = RencodeValue::Str("Green".into());

        let value = extract_single(&response).expect("extract");
        let state: SchedulerState = SchedulerState::deserialize(&value).expect("deserialize");

        assert_eq!(state, SchedulerState::Green);
    }
}
