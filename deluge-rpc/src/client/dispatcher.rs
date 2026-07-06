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

    pub async fn dispatch(
        &self,
        request: DelugeRpcRequest,
    ) -> Result<RencodeValue, DelugeRpcError> {
        let deadline = timeout(self.info.rpc_timeout, async {
            let connection = self.manager.acquire().await?;
            connection.send(request).await
        });
        deadline.await.map_err(|_| DelugeRpcError::Timeout)?
    }
}
