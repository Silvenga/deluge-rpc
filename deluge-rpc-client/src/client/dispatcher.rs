use crate::client::info::DelugeConnectionInfo;
use crate::client::manager::ConnectionManager;
use crate::{DelugeRpcError, DelugeRpcRequest, RencodeValue};
use std::sync::Arc;
use tokio::time::timeout;

#[derive(Clone)]
pub struct DelugeClientDispatcher {
    manager: Arc<ConnectionManager>,
    info: Arc<DelugeConnectionInfo>,
}

impl DelugeClientDispatcher {
    pub fn new(info: Arc<DelugeConnectionInfo>) -> Self {
        Self {
            manager: ConnectionManager::new(info.clone()).into(),
            info,
        }
    }

    pub async fn is_connected(&self) -> bool {
        self.manager.is_connected().await
    }

    pub async fn dispatch(
        &self,
        request: DelugeRpcRequest,
    ) -> Result<RencodeValue, DelugeRpcError> {
        #[cfg(feature = "recorder")]
        let request_clone = request.clone();

        let deadline = timeout(self.info.rpc_timeout, async {
            let connection = self.manager.acquire().await?;
            connection.send(request).await
        });
        let result = deadline.await.map_err(|_| DelugeRpcError::Timeout)?;

        #[cfg(feature = "recorder")]
        {
            use crate::recorder::RecordedInteraction;

            if let Some(tx) = &self.info.recorder_tx {
                if let Some(interaction) = RecordedInteraction::from_result(request_clone, &result)
                {
                    if let Err(e) = tx.try_send(interaction) {
                        tracing::warn!(error = %e, "failed to send recorded interaction");
                    }
                }
            }
        }

        result
    }
}
