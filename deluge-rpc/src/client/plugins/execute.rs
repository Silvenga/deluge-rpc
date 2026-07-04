use crate::client::RpcCaller;
use crate::models::plugins::{ExecuteCommand, ExecuteEvent};
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use crate::rencode::{RencodeValue, to_rencode_value};
use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait ExecuteRpc: Send + Sync {
    async fn add_command(&self, event: &ExecuteEvent, command: &str) -> anyhow::Result<()>;
    async fn get_commands(&self) -> anyhow::Result<Vec<ExecuteCommand>>;
    async fn remove_command(&self, command_id: &str) -> anyhow::Result<()>;
    async fn save_command(&self, command_id: &str, event: &ExecuteEvent, command: &str) -> anyhow::Result<()>;
}

pub struct ExecuteClient {
    caller: RpcCaller,
}

impl ExecuteClient {
    pub(crate) fn new(caller: RpcCaller) -> Self {
        Self { caller }
    }
}

impl Clone for ExecuteClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl ExecuteRpc for ExecuteClient {
    async fn add_command(&self, event: &ExecuteEvent, command: &str) -> anyhow::Result<()> {
        let event_value = to_rencode_value(event).context("serializing execute event")?;
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("execute.add_command").with_args(vec![
                    event_value,
                    RencodeValue::Str(command.to_owned()),
                ]),
            )
            .await?;
        Ok(())
    }

    async fn get_commands(&self) -> anyhow::Result<Vec<ExecuteCommand>> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("execute.get_commands"))
            .await
            .context("execute.get_commands RPC failed")?;
        let value = extract_single(&result, "execute.get_commands")?;
        Vec::<ExecuteCommand>::deserialize(&value).context("deserializing execute commands")
    }

    async fn remove_command(&self, command_id: &str) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("execute.remove_command").with_args(vec![
                    RencodeValue::Str(command_id.to_owned()),
                ]),
            )
            .await?;
        Ok(())
    }

    async fn save_command(&self, command_id: &str, event: &ExecuteEvent, command: &str) -> anyhow::Result<()> {
        let event_value = to_rencode_value(event).context("serializing execute event")?;
        self.caller
            .rpc_call(
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
    use crate::rencode::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_execute_get_commands_response_then_deserializes() {
        let response = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::List(vec![
                RencodeValue::Str("abc123".into()),
                RencodeValue::Str("complete".into()),
                RencodeValue::Str("echo done".into()),
            ]),
        ])]);

        let value = extract_single(&response, "execute.get_commands").expect("extract");
        let commands: Vec<ExecuteCommand> = Vec::<ExecuteCommand>::deserialize(&value).expect("deserialize");

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].command_id, "abc123");
        assert_eq!(commands[0].event, ExecuteEvent::Complete);
        assert_eq!(commands[0].command, "echo done");
    }
}
