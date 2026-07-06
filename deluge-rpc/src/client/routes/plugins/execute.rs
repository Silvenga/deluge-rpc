use crate::DelugeRpcError;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{ExecuteCommand, ExecuteEvent};
use crate::protocol::{DelugeRpcRequest, extract_single};
use crate::{RencodeValue, to_rencode_value};
use async_trait::async_trait;
use serde::Deserialize;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait ExecuteRpc: Send + Sync {
    async fn add_command(&self, event: &ExecuteEvent, command: &str) -> Result<(), DelugeRpcError>;
    async fn get_commands(&self) -> Result<Vec<ExecuteCommand>, DelugeRpcError>;
    async fn remove_command(&self, command_id: &str) -> Result<(), DelugeRpcError>;
    async fn save_command(
        &self,
        command_id: &str,
        event: &ExecuteEvent,
        command: &str,
    ) -> Result<(), DelugeRpcError>;
}

pub struct ExecuteClient {
    dispatcher: DelugeClientDispatcher,
}

impl ExecuteClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for ExecuteClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl ExecuteRpc for ExecuteClient {
    async fn add_command(&self, event: &ExecuteEvent, command: &str) -> Result<(), DelugeRpcError> {
        let event_value = to_rencode_value(event)?;
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("execute.add_command")
                    .with_args(vec![event_value, RencodeValue::Str(command.to_owned())]),
            )
            .await?;
        Ok(())
    }

    async fn get_commands(&self) -> Result<Vec<ExecuteCommand>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("execute.get_commands"))
            .await?;
        let value = extract_single(&result)?;
        Ok(Vec::<ExecuteCommand>::deserialize(&value)?)
    }

    async fn remove_command(&self, command_id: &str) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("execute.remove_command")
                    .with_args(vec![RencodeValue::Str(command_id.to_owned())]),
            )
            .await?;
        Ok(())
    }

    async fn save_command(
        &self,
        command_id: &str,
        event: &ExecuteEvent,
        command: &str,
    ) -> Result<(), DelugeRpcError> {
        let event_value = to_rencode_value(event)?;
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("execute.save_command").with_args(vec![
                    RencodeValue::Str(command_id.to_owned()),
                    event_value,
                    RencodeValue::Str(command.to_owned()),
                ]),
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_execute_get_commands_response_then_deserializes() {
        let response = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Str("abc123".into()),
            RencodeValue::Str("complete".into()),
            RencodeValue::Str("echo done".into()),
        ])]);

        let value = extract_single(&response).expect("extract");
        let commands: Vec<ExecuteCommand> =
            Vec::<ExecuteCommand>::deserialize(&value).expect("deserialize");

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].command_id, "abc123");
        assert_eq!(commands[0].event, ExecuteEvent::Complete);
        assert_eq!(commands[0].command, "echo done");
    }
}
