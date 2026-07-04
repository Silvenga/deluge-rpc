use crate::models::TorrentInfo;
use crate::protocol::DelugeRpcMessage;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::{extract_single, extract_single_dict, extract_single_int};
use crate::rencode::RencodeValue;
use crate::rpc::DelugeRpc;
use crate::shared::Shared;
use crate::transport::DelugeWriter;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{Mutex, broadcast};
use tokio::time::timeout;

const RPC_TIMEOUT: Duration = Duration::from_secs(30);

pub struct DelugeRpcClient {
    shared: Arc<Shared>,
    writer: Arc<Mutex<DelugeWriter>>,
}

impl DelugeRpcClient {
    pub(crate) fn new(shared: Arc<Shared>, writer: Arc<Mutex<DelugeWriter>>) -> Self {
        Self { shared, writer }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<DelugeRpcMessage> {
        self.shared.message_tx.subscribe()
    }

    fn next_id(&self) -> u32 {
        self.shared.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn rpc_call(&self, request: DelugeRpcRequest) -> anyhow::Result<RencodeValue> {
        let id = self.next_id();
        let method = request.method.clone();
        let encoded = request.encode(id);

        let mut rx = self.shared.message_tx.subscribe();

        {
            let mut writer = self.writer.lock().await;
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
}

impl Clone for DelugeRpcClient {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
            writer: Arc::clone(&self.writer),
        }
    }
}

#[async_trait]
impl DelugeRpc for DelugeRpcClient {
    async fn get_free_space(&self) -> anyhow::Result<u64> {
        let result = self
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
