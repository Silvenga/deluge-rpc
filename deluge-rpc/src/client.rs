use crate::models::TorrentInfo;
use crate::rencode::RencodeValue;
use crate::rpc::DelugeRpc;
use crate::transport::DelugeTransport;
use crate::wire::{
    ResponseOutcome, build_request, extract_single, extract_single_dict, extract_single_int,
    handle_response,
};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;

pub struct DelugeRpcClient {
    host: String,
    port: u16,
    username: String,
    password: String,
    transport: Mutex<Option<DelugeTransport>>,
    next_request_id: AtomicU32,
}

impl DelugeRpcClient {
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
            transport: Mutex::new(None),
            next_request_id: AtomicU32::new(1),
        }
    }

    fn next_id(&self) -> u32 {
        self.next_request_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn rpc_call(
        &self,
        method: &str,
        args: Vec<RencodeValue>,
        kwargs: BTreeMap<RencodeValue, RencodeValue>,
    ) -> anyhow::Result<RencodeValue> {
        let id = self.next_id();
        let encoded = build_request(id, method, args, kwargs).encode();

        let mut guard = self.transport.lock().await;
        let transport = guard
            .as_mut()
            .ok_or_else(|| anyhow!("not connected — call login() first"))?;
        transport
            .send(&encoded)
            .await
            .with_context(|| format!("failed to send RPC request `{method}`"))?;

        loop {
            let raw = timeout(Duration::from_secs(30), transport.recv())
                .await
                .map_err(|_| anyhow!("timed out waiting for RPC response `{method}`"))?
                .with_context(|| format!("failed to recv RPC response for `{method}`"))?;
            let decoded = RencodeValue::decode(&raw)
                .with_context(|| format!("failed to decode RPC response for `{method}`"))?;

            match handle_response(&decoded, id, method)? {
                ResponseOutcome::Return(value) => return Ok(value),
                ResponseOutcome::Continue => {}
            }
        }
    }
}

#[async_trait]
impl DelugeRpc for DelugeRpcClient {
    async fn login(&self) -> anyhow::Result<()> {
        let transport = DelugeTransport::connect(&self.host, self.port)
            .await
            .context("failed to connect to Deluge daemon")?;
        *self.transport.lock().await = Some(transport);

        let mut kwargs = BTreeMap::new();
        kwargs.insert(
            RencodeValue::Str(String::from("client_version")),
            RencodeValue::Str(String::from(concat!(
                "deluge-retain/",
                env!("CARGO_PKG_VERSION")
            ))),
        );

        let args = vec![
            RencodeValue::Str(self.username.clone()),
            RencodeValue::Str(self.password.clone()),
        ];

        let result = self
            .rpc_call("daemon.login", args, kwargs)
            .await
            .context("daemon.login RPC failed")?;

        let auth_level = extract_single_int(&result, "daemon.login")?;
        tracing::debug!(auth_level, "daemon.login succeeded");
        Ok(())
    }

    async fn get_free_space(&self) -> anyhow::Result<u64> {
        let result = self
            .rpc_call(
                "core.get_free_space",
                vec![RencodeValue::None],
                BTreeMap::new(),
            )
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

        // NOTE: filter_dict is FIRST, keys is SECOND — reversed from web.update_ui.
        let args = vec![
            RencodeValue::Dict(BTreeMap::default()),
            RencodeValue::List(keys),
        ];

        let result = self
            .rpc_call("core.get_torrents_status", args, BTreeMap::default())
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
        let args = vec![
            RencodeValue::Str(id.to_owned()),
            RencodeValue::Bool(true),
        ];

        let result = self
            .rpc_call("core.remove_torrent", args, BTreeMap::default())
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
