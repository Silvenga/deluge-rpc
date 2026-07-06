use crate::DelugeRpcError;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{SchedulerConfig, SchedulerState};
use crate::protocol::{DelugeRpcRequest, extract_single};
use crate::to_rencode_value;
use async_trait::async_trait;
use serde::Deserialize;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait SchedulerRpc: Send + Sync {
    async fn set_config(&self, config: &SchedulerConfig) -> Result<(), DelugeRpcError>;
    async fn get_config(&self) -> Result<SchedulerConfig, DelugeRpcError>;
    async fn get_state(&self) -> Result<SchedulerState, DelugeRpcError>;
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
    async fn set_config(&self, config: &SchedulerConfig) -> Result<(), DelugeRpcError> {
        let config_value = to_rencode_value(config)?;
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("scheduler.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> Result<SchedulerConfig, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("scheduler.get_config"))
            .await?;
        let value = extract_single(&result)?;
        Ok(SchedulerConfig::deserialize(&value)?)
    }

    async fn get_state(&self) -> Result<SchedulerState, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("scheduler.get_state"))
            .await?;
        let value = extract_single(&result)?;
        Ok(SchedulerState::deserialize(&value)?)
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
