use crate::client::dispatch::dispatch;
use crate::client::manager::ConnectionManager;
use crate::{DelugeRpcRequest, RencodeValue};
use anyhow::Context;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

const RPC_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct DelugeClientDispatcher {
    manager: Arc<ConnectionManager>,
}

impl DelugeClientDispatcher {
    pub fn new(manager: ConnectionManager) -> Self {
        Self {
            manager: manager.into(),
        }
    }

    pub async fn dispatch(&self, request: DelugeRpcRequest) -> anyhow::Result<RencodeValue> {
        let deadline = timeout(RPC_TIMEOUT, async {
            let connection = self
                .manager
                .acquire()
                .await
                .context("failed to acquire connection")?;

            dispatch(&connection, request).await
        });
        deadline.await.expect("timed out waiting for RPC response")
    }
}
