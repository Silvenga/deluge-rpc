use crate::DelugeRpcError;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{HandledEvent, NotificationsConfig};
use crate::protocol::{DelugeRpcRequest, extract_single};
use crate::to_rencode_value;

use serde::Deserialize;

/// RPC methods for the `notifications.*` namespace.
pub trait NotificationsRpc: Send + Sync {
    /// Sets the plugin config.
    async fn set_config(&self, config: &NotificationsConfig) -> Result<(), DelugeRpcError>;
    /// Returns the plugin config.
    async fn get_config(&self) -> Result<NotificationsConfig, DelugeRpcError>;
    /// Returns events that the plugin can handle.
    async fn get_handled_events(&self) -> Result<Vec<HandledEvent>, DelugeRpcError>;
}

/// Client for `notifications.*` RPC methods.
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

impl NotificationsRpc for NotificationsClient {
    async fn set_config(&self, config: &NotificationsConfig) -> Result<(), DelugeRpcError> {
        let config_value = to_rencode_value(config)?;
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("notifications.set_config").with_args(vec![config_value]),
            )
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> Result<NotificationsConfig, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("notifications.get_config"))
            .await?;
        let value = extract_single(&result)?;
        Ok(NotificationsConfig::deserialize(&value)?)
    }

    async fn get_handled_events(&self) -> Result<Vec<HandledEvent>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("notifications.get_handled_events"))
            .await?;
        let value = extract_single(&result)?;
        Ok(Vec::<HandledEvent>::deserialize(&value)?)
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
