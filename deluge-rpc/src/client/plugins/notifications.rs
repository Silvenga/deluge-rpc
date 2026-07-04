use crate::client::RpcCaller;
use crate::models::plugins::{HandledEvent, NotificationsConfig};
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
pub trait NotificationsRpc: Send + Sync {
    async fn set_config(&self, config: &NotificationsConfig) -> anyhow::Result<()>;
    async fn get_config(&self) -> anyhow::Result<NotificationsConfig>;
    async fn get_handled_events(&self) -> anyhow::Result<Vec<HandledEvent>>;
}

pub struct NotificationsClient {
    caller: RpcCaller,
}

impl NotificationsClient {
    pub(crate) fn new(shared: Arc<Shared>, writer: Arc<Mutex<DelugeWriter>>) -> Self {
        Self {
            caller: RpcCaller::new(shared, writer),
        }
    }
}

impl Clone for NotificationsClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl NotificationsRpc for NotificationsClient {
    async fn set_config(&self, config: &NotificationsConfig) -> anyhow::Result<()> {
        let config_value = to_rencode_value(config).context("serializing notifications config")?;
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("notifications.set_config").with_args(vec![config_value]),
            )
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> anyhow::Result<NotificationsConfig> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("notifications.get_config"))
            .await
            .context("notifications.get_config RPC failed")?;
        let value = extract_single(&result, "notifications.get_config")?;
        NotificationsConfig::deserialize(&value).context("deserializing notifications config")
    }

    async fn get_handled_events(&self) -> anyhow::Result<Vec<HandledEvent>> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("notifications.get_handled_events"))
            .await
            .context("notifications.get_handled_events RPC failed")?;
        let value = extract_single(&result, "notifications.get_handled_events")?;
        Vec::<HandledEvent>::deserialize(&value).context("deserializing handled events")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_notifications_get_handled_events_response_then_deserializes() {
        let response = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::List(vec![
                RencodeValue::Str("TorrentFinishedEvent".into()),
                RencodeValue::Str("Emitted when a torrent finishes downloading.".into()),
            ]),
        ])]);

        let value = extract_single(&response, "notifications.get_handled_events").expect("extract");
        let events: Vec<HandledEvent> = Vec::<HandledEvent>::deserialize(&value).expect("deserialize");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_name, "TorrentFinishedEvent");
    }
}
