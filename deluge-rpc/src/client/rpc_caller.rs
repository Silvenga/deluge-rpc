use crate::protocol::DelugeRpcMessage;
use crate::protocol::DelugeRpcRequest;
use crate::rencode::RencodeValue;
use anyhow::{Context, anyhow};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::timeout;

use super::deluge_client::{ConnectionState, DelugeClientInner};

const RPC_TIMEOUT: Duration = Duration::from_secs(30);

pub struct RpcCaller {
    inner: Arc<DelugeClientInner>,
}

impl RpcCaller {
    pub(crate) fn new_reconnect(inner: Arc<DelugeClientInner>) -> Self {
        Self { inner }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<DelugeRpcMessage> {
        match self.inner.state.try_lock() {
            Ok(state) => match &*state {
                ConnectionState::Connected { shared, .. } => shared.message_tx.subscribe(),
                ConnectionState::Disconnected => {
                    let (tx, rx) = broadcast::channel(1);
                    drop(tx);
                    rx
                }
            },
            Err(_) => {
                let (tx, rx) = broadcast::channel(1);
                drop(tx);
                rx
            }
        }
    }

    pub async fn rpc_call(&self, request: DelugeRpcRequest) -> anyhow::Result<RencodeValue> {
        rpc_call_reconnect(&self.inner, request).await
    }
}

impl Clone for RpcCaller {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

async fn rpc_call_reconnect(
    inner: &DelugeClientInner,
    request: DelugeRpcRequest,
) -> anyhow::Result<RencodeValue> {
    let method = request.method.clone();

    let (writer, mut rx) = {
        let mut state = inner.state.lock().await;
        state
            .ensure_connected(&inner.host, inner.port, &inner.username, &inner.password)
            .await
            .with_context(|| format!("reconnect failed for RPC `{method}`"))?;
        state
            .writer_and_rx()
            .ok_or_else(|| anyhow!("connection lost after reconnect for RPC `{method}`"))?
    };

    let id = inner.next_id.fetch_add(1, Ordering::Relaxed);
    let encoded = request.encode(id);

    {
        let mut writer = writer.lock().await;
        if let Err(e) = writer.send(&encoded).await {
            let mut state = inner.state.lock().await;
            *state = ConnectionState::Disconnected;
            return Err(anyhow!("failed to send RPC request `{method}`: {e}"));
        }
    }

    let deadline = timeout(RPC_TIMEOUT, async {
        loop {
            match rx.recv().await {
                Ok(DelugeRpcMessage::Response { id: resp_id, value }) if resp_id == id => {
                    return Ok(value);
                }
                Ok(DelugeRpcMessage::Error {
                    id: err_id,
                    exc_type,
                    exc_msg,
                    traceback,
                }) if err_id == id => {
                    return Err(anyhow!(
                        "daemon RPC error ({exc_type}): {exc_msg}\ntraceback: {traceback}"
                    ));
                }
                Ok(_) => continue,
                Err(RecvError::Lagged(n)) => {
                    tracing::warn!(n, "RPC subscriber lagged, missed messages");
                    continue;
                }
                Err(RecvError::Closed) => {
                    return Err(anyhow!(
                        "connection closed while waiting for RPC response `{method}`"
                    ));
                }
            }
        }
    })
    .await;

    match deadline {
        Ok(result) => result,
        Err(_) => {
            let mut state = inner.state.lock().await;
            *state = ConnectionState::Disconnected;
            Err(anyhow!("timed out waiting for RPC response `{method}`"))
        }
    }
}
