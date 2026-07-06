use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{HandledEvent, NotificationsConfig};
use crate::protocol::{extract_single, DelugeRpcRequest};
use crate::to_rencode_value;
use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait NotificationsRpc: Send + Sync {
    async fn set_config(&self, config: &NotificationsConfig) -> anyhow::Result<()>;
    async fn get_config(&self) -> anyhow::Result<NotificationsConfig>;
    async fn get_handled_events(&self) -> anyhow::Result<Vec<HandledEvent>>;
}

pub struct NotificationsClient {
    dispatcher: DelugeClientDispatcher,
}

impl NotificationsClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for NotificationsClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl NotificationsRpc for NotificationsClient {
    async fn set_config(&self, config: &NotificationsConfig) -> anyhow::Result<()> {
        let config_value = to_rencode_value(config).context("serializing notifications config")?;
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("notifications.set_config").with_args(vec![config_value]),
            )
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> anyhow::Result<NotificationsConfig> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("notifications.get_config"))
            .await
            .context("notifications.get_config RPC failed")?;
        let value = extract_single(&result)?;
        NotificationsConfig::deserialize(&value).context("deserializing notifications config")
    }

    async fn get_handled_events(&self) -> anyhow::Result<Vec<HandledEvent>> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("notifications.get_handled_events"))
            .await
            .context("notifications.get_handled_events RPC failed")?;
        let value = extract_single(&result)?;
        Vec::<HandledEvent>::deserialize(&value).context("deserializing handled events")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_notifications_get_handled_events_response_then_deserializes() {
        let response = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Str("TorrentFinishedEvent".into()),
            RencodeValue::Str("Emitted when a torrent finishes downloading.".into()),
        ])]);

        let value = extract_single(&response).expect("extract");
        let events: Vec<HandledEvent> =
            Vec::<HandledEvent>::deserialize(&value).expect("deserialize");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_name, "TorrentFinishedEvent");
    }
}
