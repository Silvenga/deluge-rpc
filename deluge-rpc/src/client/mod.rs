pub mod core;
pub mod daemon;
pub mod deluge_client;
pub mod plugins;

use crate::protocol::DelugeRpcMessage;
use crate::protocol::DelugeRpcRequest;
use crate::rencode::RencodeValue;
use crate::shared::Shared;
use crate::transport::DelugeWriter;
use anyhow::{Context, anyhow};
use deluge_client::{ConnectionState, DelugeClientInner};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{Mutex, broadcast};
use tokio::time::timeout;

const RPC_TIMEOUT: Duration = Duration::from_secs(30);

enum RpcCallerBackend {
    Direct {
        shared: Arc<Shared>,
        writer: Arc<Mutex<DelugeWriter>>,
    },
    Reconnect {
        inner: Arc<DelugeClientInner>,
    },
}

pub struct RpcCaller {
    backend: RpcCallerBackend,
}

impl RpcCaller {
    pub(crate) fn new(shared: Arc<Shared>, writer: Arc<Mutex<DelugeWriter>>) -> Self {
        Self {
            backend: RpcCallerBackend::Direct { shared, writer },
        }
    }

    pub(crate) fn new_reconnect(inner: Arc<DelugeClientInner>) -> Self {
        Self {
            backend: RpcCallerBackend::Reconnect { inner },
        }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<DelugeRpcMessage> {
        match &self.backend {
            RpcCallerBackend::Direct { shared, .. } => shared.message_tx.subscribe(),
            RpcCallerBackend::Reconnect { inner } => {
                match inner.state.try_lock() {
                    Ok(state) => match &*state {
                        deluge_client::ConnectionState::Connected { shared, .. } => {
                            shared.message_tx.subscribe()
                        }
                        deluge_client::ConnectionState::Disconnected => {
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
        }
    }

    pub async fn rpc_call(&self, request: DelugeRpcRequest) -> anyhow::Result<RencodeValue> {
        match &self.backend {
            RpcCallerBackend::Direct { shared, writer } => {
                rpc_call_direct(shared, writer, request).await
            }
            RpcCallerBackend::Reconnect { inner } => {
                rpc_call_reconnect(inner, request).await
            }
        }
    }
}

impl Clone for RpcCaller {
    fn clone(&self) -> Self {
        match &self.backend {
            RpcCallerBackend::Direct { shared, writer } => Self {
                backend: RpcCallerBackend::Direct {
                    shared: Arc::clone(shared),
                    writer: Arc::clone(writer),
                },
            },
            RpcCallerBackend::Reconnect { inner } => Self {
                backend: RpcCallerBackend::Reconnect {
                    inner: Arc::clone(inner),
                },
            },
        }
    }
}

async fn rpc_call_direct(
    shared: &Shared,
    writer: &Arc<Mutex<DelugeWriter>>,
    request: DelugeRpcRequest,
) -> anyhow::Result<RencodeValue> {
    let id = shared.next_id.fetch_add(1, Ordering::Relaxed);
    let method = request.method.clone();
    let encoded = request.encode(id);

    let mut rx = shared.message_tx.subscribe();

    {
        let mut writer = writer.lock().await;
        writer
            .send(&encoded)
            .await
            .with_context(|| format!("failed to send RPC request `{method}`"))?;
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

    deadline.unwrap_or_else(|_| Err(anyhow!("timed out waiting for RPC response `{method}`")))
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

// --- Old DelugeRpcClient (compat shim, removed in task 11) ---

use crate::models::TorrentInfo;
use crate::protocol::{extract_single, extract_single_dict, extract_single_int};
use crate::rpc::DelugeRpc;
use async_trait::async_trait;
use std::collections::BTreeMap;

pub struct DelugeRpcClient {
    caller: RpcCaller,
}

impl DelugeRpcClient {
    pub(crate) fn new(shared: Arc<Shared>, writer: Arc<Mutex<DelugeWriter>>) -> Self {
        Self {
            caller: RpcCaller::new(shared, writer),
        }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<DelugeRpcMessage> {
        self.caller.subscribe_events()
    }

    pub async fn rpc_call(&self, request: DelugeRpcRequest) -> anyhow::Result<RencodeValue> {
        self.caller.rpc_call(request).await
    }
}

impl Clone for DelugeRpcClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl DelugeRpc for DelugeRpcClient {
    async fn get_free_space(&self) -> anyhow::Result<u64> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_free_space"))
            .await
            .context("core.get_free_space RPC failed")?;

        let bytes = extract_single_int(&result, "core.get_free_space")?;
        u64::try_from(bytes)
            .map_err(|_| anyhow!("core.get_free_space returned negative value: {bytes}"))
    }

    async fn get_torrents(&self) -> anyhow::Result<Vec<TorrentInfo>> {
        let keys = vec![
            RencodeValue::Str(String::from("name")),
            RencodeValue::Str(String::from("state")),
            RencodeValue::Str(String::from("progress")),
            RencodeValue::Str(String::from("ratio")),
            RencodeValue::Str(String::from("total_seeds")),
            RencodeValue::Str(String::from("num_seeds")),
            RencodeValue::Str(String::from("time_added")),
            RencodeValue::Str(String::from("total_done")),
            RencodeValue::Str(String::from("total_uploaded")),
            RencodeValue::Str(String::from("is_finished")),
            RencodeValue::Str(String::from("download_location")),
        ];

        let args = vec![
            RencodeValue::Dict(BTreeMap::default()),
            RencodeValue::List(keys),
        ];

        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_torrents_status").with_args(args))
            .await
            .context("core.get_torrents_status RPC failed")?;

        let result_dict = extract_single_dict(&result, "core.get_torrents_status")?;

        let mut entries: Vec<(String, &RencodeValue)> = result_dict
            .iter()
            .filter_map(|(k, v)| match (k, v) {
                (RencodeValue::Str(id), fields) => Some((id.clone(), fields)),
                _ => None,
            })
            .collect();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut out = Vec::with_capacity(entries.len());
        for (info_hash, fields) in entries {
            let info = TorrentInfo::from(&info_hash, fields)
                .with_context(|| format!("parsing torrent `{info_hash}`"))?;
            out.push(info);
        }
        Ok(out)
    }

    async fn remove_torrent(&self, id: &str) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.remove_torrent").with_args(vec![
                RencodeValue::Str(id.to_owned()),
                RencodeValue::Bool(true),
            ]))
            .await
            .context("core.remove_torrent RPC failed")?;

        let value = extract_single(&result, "core.remove_torrent")?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "core.remove_torrent returned non-bool value: {other:?}"
            )),
        }
    }
}
