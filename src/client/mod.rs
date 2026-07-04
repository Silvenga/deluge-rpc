//! Deluge daemon RPC client.
//!
//! Speaks the native Deluge daemon protocol (TLS + framed zlib + rencode)
//! instead of the Web UI JSON-RPC API. The client holds a single
//! [`DelugeTransport`] that is established on the first `login` call
//! and dropped when the [`DelugeClient`] is dropped — one connection per
//! retain cycle, no persistence.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::rencode::{decode, encode, RencodeValue};
use crate::torrent::TorrentInfo;
use crate::transport::DelugeTransport;

/// RPC response message type tag (Deluge daemon protocol).
const RPC_RESPONSE: i64 = 1;
/// RPC error message type tag (Deluge daemon protocol).
const RPC_ERROR: i64 = 2;
/// RPC event message type tag (Deluge daemon protocol).
const RPC_EVENT: i64 = 3;

/// Deluge daemon RPC surface used by the engine and main.
///
/// Extracted from [`DelugeClient`] so callers can be tested against a
/// mockall-generated `MockDelugeRpc` instead of a live daemon connection.
/// The concrete [`DelugeClient`] is the only production implementation.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DelugeRpc: Send + Sync {
    /// Authenticate against the daemon via `daemon.login`.
    async fn login(&self) -> anyhow::Result<()>;

    /// Query free disk space in bytes at the daemon's download location.
    async fn get_free_space(&self) -> anyhow::Result<u64>;

    /// Fetch the current status of all torrents.
    async fn get_torrents(&self) -> anyhow::Result<Vec<TorrentInfo>>;

    /// Remove a torrent by info hash (with `remove_data = true`).
    async fn remove_torrent(&self, id: &str) -> anyhow::Result<bool>;
}

/// Async client for the Deluge daemon RPC protocol.
///
/// The transport is `None` until [`DelugeRpc::login`] connects; all
/// subsequent method calls reuse the established connection. Interior
/// mutability via a [`tokio::sync::Mutex`] guards the `Option` slot —
/// the async transport calls happen outside the lock.
pub struct DelugeClient {
    host: String,
    port: u16,
    username: String,
    password: String,
    transport: Mutex<Option<DelugeTransport>>,
    request_id: AtomicU32,
}

impl DelugeClient {
    /// Create a new client targeting `host:port` that authenticates with
    /// `username` / `password`. No connection is opened until
    /// [`DelugeRpc::login`] is called.
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
            transport: Mutex::new(None),
            request_id: AtomicU32::new(1),
        }
    }

    fn next_id(&self) -> u32 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Core RPC dispatcher.
    ///
    /// Builds the request `[[request_id, method, [args], {kwargs}]]`,
    /// encodes it via rencode, sends it over the transport, and reads
    /// responses until one with the matching `request_id` arrives. RPC
    /// events (type 3) are logged at trace level and skipped.
    async fn rpc_call(
        &self,
        method: &str,
        args: Vec<RencodeValue>,
        kwargs: BTreeMap<RencodeValue, RencodeValue>,
    ) -> anyhow::Result<RencodeValue> {
        let id = self.next_id();
        let request = build_request(id, method, args, kwargs);
        let encoded = encode(&request);

        let mut guard = self.transport.lock().await;
        let transport = guard
            .as_mut()
            .ok_or_else(|| anyhow!("not connected — call login() first"))?;
        transport
            .send(&encoded)
            .await
            .with_context(|| format!("failed to send RPC request `{method}`"))?;

        loop {
            let raw = transport
                .recv()
                .await
                .with_context(|| format!("failed to recv RPC response for `{method}`"))?;
            let decoded = decode(&raw)
                .with_context(|| format!("failed to decode RPC response for `{method}`"))?;

            match handle_response(&decoded, id, method)? {
                ResponseOutcome::Return(value) => return Ok(value),
                ResponseOutcome::Continue => {}
            }
        }
    }
}

#[async_trait]
impl DelugeRpc for DelugeClient {
    /// Authenticate against the daemon via `daemon.login`.
    ///
    /// Establishes the TLS transport on first call. Subsequent method
    /// calls reuse the connection. The returned auth level is logged at
    /// debug level and otherwise ignored.
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

    /// Query free disk space in bytes at the daemon's default download
    /// location via `core.get_free_space`.
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

    /// Fetch the current status of all torrents via
    /// `core.get_torrents_status`.
    ///
    /// Requests the explicit field set required to populate
    /// [`TorrentInfo`]. The daemon returns a dict keyed by info hash,
    /// which is flattened into a [`Vec<TorrentInfo>`] sorted by info
    /// hash for deterministic output.
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

        // NOTE: filter_dict is FIRST, keys is SECOND — reversed from
        // web.update_ui.
        let args = vec![
            RencodeValue::Dict(BTreeMap::new()),
            RencodeValue::List(keys),
        ];

        let result = self
            .rpc_call("core.get_torrents_status", args, BTreeMap::new())
            .await
            .context("core.get_torrents_status RPC failed")?;

        let result_dict = extract_single_dict(&result, "core.get_torrents_status")?;

        let mut entries: Vec<(String, &BTreeMap<RencodeValue, RencodeValue>)> = result_dict
            .iter()
            .filter_map(|(k, v)| match (k, v) {
                (RencodeValue::Str(id), RencodeValue::Dict(fields)) => Some((id.clone(), fields)),
                _ => None,
            })
            .collect();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut out = Vec::with_capacity(entries.len());
        for (info_hash, fields) in entries {
            let info = parse_torrent(&info_hash, fields)
                .with_context(|| format!("parsing torrent `{info_hash}`"))?;
            out.push(info);
        }
        Ok(out)
    }

    /// Remove a torrent by info hash. `remove_data = true` deletes the
    /// downloaded files as well.
    async fn remove_torrent(&self, id: &str) -> anyhow::Result<bool> {
        let args = vec![
            RencodeValue::Str(String::from(id)),
            RencodeValue::Bool(true),
        ];

        let result = self
            .rpc_call("core.remove_torrent", args, BTreeMap::new())
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

/// Build the rencode request envelope `[[id, method, [args], {kwargs}]]`.
fn build_request(
    id: u32,
    method: &str,
    args: Vec<RencodeValue>,
    kwargs: BTreeMap<RencodeValue, RencodeValue>,
) -> RencodeValue {
    RencodeValue::List(vec![RencodeValue::List(vec![
        RencodeValue::Int(i64::from(id)),
        RencodeValue::Str(String::from(method)),
        RencodeValue::List(args),
        RencodeValue::Dict(kwargs),
    ])])
}

/// Outcome of inspecting one decoded response message.
#[derive(Debug)]
enum ResponseOutcome {
    /// A matching response was found; return this value to the caller.
    Return(RencodeValue),
    /// An event or mismatched-id response was seen; read the next message.
    Continue,
}

/// Inspect a decoded response and decide whether to return or keep reading.
fn handle_response(
    decoded: &RencodeValue,
    expected_id: u32,
    method: &str,
) -> anyhow::Result<ResponseOutcome> {
    let outer = match decoded {
        RencodeValue::List(items) if items.len() == 1 => {
            #[expect(
                clippy::expect_used,
                reason = "len == 1 is checked by the guard; first() cannot be None"
            )]
            items.first().expect("len == 1 checked above")
        }
        other => {
            return Err(anyhow!(
                "unexpected RPC envelope shape (not a 1-element list): {other:?}"
            ))
        }
    };

    let inner = match outer {
        RencodeValue::List(parts) => parts,
        other => {
            return Err(anyhow!(
                "unexpected RPC message shape (not a list): {other:?}"
            ))
        }
    };

    if inner.is_empty() {
        return Err(anyhow!("empty RPC message"));
    }

    let msg_type = match inner.first() {
        Some(RencodeValue::Int(t)) => *t,
        Some(other) => {
            return Err(anyhow!("RPC message type tag is not an int: {other:?}"))
        }
        None => return Err(anyhow!("RPC message missing type tag")),
    };

    match msg_type {
        RPC_RESPONSE => extract_response_value(inner, expected_id, method),
        RPC_ERROR => Err(rpc_error(inner)),
        RPC_EVENT => {
            let event_name =
                field_as_str(inner.get(1)).unwrap_or_else(|| "<unknown>".to_owned());
            tracing::trace!(event = %event_name, "daemon RPC event");
            Ok(ResponseOutcome::Continue)
        }
        other => Err(anyhow!(
            "unexpected RPC message type {other} (expected 1/2/3)"
        )),
    }
}

/// Extract the return value from an `RPC_RESPONSE` message, validating the id.
fn extract_response_value(
    inner: &[RencodeValue],
    expected_id: u32,
    method: &str,
) -> anyhow::Result<ResponseOutcome> {
    let resp_id = match inner.get(1) {
        Some(RencodeValue::Int(i)) => *i,
        Some(other) => {
            return Err(anyhow!("RPC response id is not an int: {other:?}"))
        }
        None => return Err(anyhow!("RPC response missing id")),
    };
    if resp_id != i64::from(expected_id) {
        tracing::trace!(
            resp_id,
            expected = expected_id,
            "ignoring RPC response with mismatched id"
        );
        return Ok(ResponseOutcome::Continue);
    }
    let value = inner
        .get(2)
        .cloned()
        .ok_or_else(|| anyhow!("RPC response for `{method}` missing return value list"))?;
    Ok(ResponseOutcome::Return(value))
}

/// Build an error from an `RPC_ERROR` message.
fn rpc_error(inner: &[RencodeValue]) -> anyhow::Error {
    let exc_type = field_as_str(inner.get(1)).unwrap_or_else(|| "<unknown>".to_owned());
    let exc_msg = field_as_str(inner.get(2)).unwrap_or_else(|| "<unknown>".to_owned());
    let traceback = field_as_str(inner.get(3)).unwrap_or_else(|| "<none>".to_owned());
    anyhow!("daemon RPC error ({exc_type}): {exc_msg}\ntraceback: {traceback}")
}

/// Extract the single return value from an RPC response's return list.
///
/// The daemon wraps the return value in a one-element list: `[value]`.
fn extract_single(value: &RencodeValue, method: &str) -> anyhow::Result<RencodeValue> {
    match value {
        RencodeValue::List(items) if items.len() == 1 => {
            #[expect(
                clippy::expect_used,
                reason = "len == 1 is checked by the guard; first() cannot be None"
            )]
            Ok(items.first().expect("len == 1 checked above").clone())
        }
        other => Err(anyhow!(
            "{method} returned unexpected return shape (expected 1-element list): {other:?}"
        )),
    }
}

/// Extract a single int return value.
fn extract_single_int(value: &RencodeValue, method: &str) -> anyhow::Result<i64> {
    let single = extract_single(value, method)?;
    match single {
        RencodeValue::Int(i) => Ok(i),
        other => Err(anyhow!("{method} returned non-int value: {other:?}")),
    }
}

/// Extract a single dict return value.
fn extract_single_dict<'a>(
    value: &'a RencodeValue,
    method: &str,
) -> anyhow::Result<&'a BTreeMap<RencodeValue, RencodeValue>> {
    match value {
        RencodeValue::List(items) if items.len() == 1 => match items.first() {
            Some(RencodeValue::Dict(map)) => Ok(map),
            Some(other) => Err(anyhow!("{method} returned non-dict value: {other:?}")),
            None => Err(anyhow!("{method} returned empty return list")),
        },
        other => Err(anyhow!(
            "{method} returned unexpected return shape (expected 1-element list): {other:?}"
        )),
    }
}

/// Best-effort string extraction from an RPC error field.
fn field_as_str(value: Option<&RencodeValue>) -> Option<String> {
    match value? {
        RencodeValue::Str(s) => Some(s.clone()),
        RencodeValue::Bytes(b) => String::from_utf8(b.clone()).ok(),
        RencodeValue::Int(i) => Some(i.to_string()),
        other => Some(format!("{other:?}")),
    }
}

/// Parse a single torrent status dict into [`TorrentInfo`].
fn parse_torrent(
    info_hash: &str,
    fields: &BTreeMap<RencodeValue, RencodeValue>,
) -> anyhow::Result<TorrentInfo> {
    let name = get_str(fields, "name")?.to_owned();
    let state = get_str(fields, "state")?.to_owned();
    let progress = get_float(fields, "progress")?;
    let ratio = get_float(fields, "ratio")?;
    let total_seeds = get_u32(fields, "total_seeds")?;
    let num_seeds = get_u32(fields, "num_seeds")?;
    let time_added = get_time_added(fields)?;
    let total_done = get_u64(fields, "total_done")?;
    let total_uploaded = get_u64(fields, "total_uploaded")?;
    let is_finished = get_bool(fields, "is_finished")?;
    let download_location = get_str(fields, "download_location")?.to_owned();

    Ok(TorrentInfo {
        info_hash: String::from(info_hash),
        name,
        state,
        progress,
        ratio,
        total_seeds,
        num_seeds,
        time_added,
        total_done,
        total_uploaded,
        is_finished,
        download_location,
    })
}

fn get_str<'a>(
    fields: &'a BTreeMap<RencodeValue, RencodeValue>,
    key: &str,
) -> anyhow::Result<&'a str> {
    match fields.get(&RencodeValue::Str(String::from(key))) {
        Some(RencodeValue::Str(s)) => Ok(s.as_str()),
        Some(other) => Err(anyhow!("field `{key}` is not a string: {other:?}")),
        None => Err(anyhow!("missing field `{key}`")),
    }
}

fn get_float(
    fields: &BTreeMap<RencodeValue, RencodeValue>,
    key: &str,
) -> anyhow::Result<f64> {
    match fields.get(&RencodeValue::Str(String::from(key))) {
        Some(RencodeValue::Float(f)) => Ok(*f),
        Some(RencodeValue::Int(i)) => {
            #[expect(
                clippy::cast_precision_loss,
                reason = "Deluge may send a float field as an int; widening to f64 is the intended coercion"
            )]
            #[expect(
                clippy::as_conversions,
                reason = "i64 to f64 is the intended widening conversion for numeric field coercion"
            )]
            Ok(*i as f64)
        }
        Some(other) => Err(anyhow!("field `{key}` is not a number: {other:?}")),
        None => Err(anyhow!("missing field `{key}`")),
    }
}

fn get_int(
    fields: &BTreeMap<RencodeValue, RencodeValue>,
    key: &str,
) -> anyhow::Result<i64> {
    match fields.get(&RencodeValue::Str(String::from(key))) {
        Some(RencodeValue::Int(i)) => Ok(*i),
        Some(other) => Err(anyhow!("field `{key}` is not an int: {other:?}")),
        None => Err(anyhow!("missing field `{key}`")),
    }
}

fn get_bool(
    fields: &BTreeMap<RencodeValue, RencodeValue>,
    key: &str,
) -> anyhow::Result<bool> {
    match fields.get(&RencodeValue::Str(String::from(key))) {
        Some(RencodeValue::Bool(b)) => Ok(*b),
        Some(other) => Err(anyhow!("field `{key}` is not a bool: {other:?}")),
        None => Err(anyhow!("missing field `{key}`")),
    }
}

fn get_u32(
    fields: &BTreeMap<RencodeValue, RencodeValue>,
    key: &str,
) -> anyhow::Result<u32> {
    let raw = get_int(fields, key)?;
    u32::try_from(raw)
        .map_err(|_| anyhow!("field `{key}` out of u32 range: {raw}"))
}

fn get_u64(
    fields: &BTreeMap<RencodeValue, RencodeValue>,
    key: &str,
) -> anyhow::Result<u64> {
    let raw = get_int(fields, key)?;
    u64::try_from(raw).map_err(|_| anyhow!("field `{key}` is negative: {raw}"))
}

fn get_time_added(
    fields: &BTreeMap<RencodeValue, RencodeValue>,
) -> anyhow::Result<i64> {
    get_int(fields, "time_added")
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use expect for clarity"
)]
#[expect(
    clippy::unwrap_used,
    reason = "test helpers use unwrap for clarity on known-good values"
)]
#[expect(
    clippy::indexing_slicing,
    reason = "tests index known-length encoded buffers"
)]
#[expect(
    clippy::as_conversions,
    reason = "tests cast small known values"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "test values fit target widths"
)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    // ----- RPC request encoding unit tests -----

    #[test]
    fn when_login_request_built_then_envelope_has_correct_shape() {
        let mut kwargs = BTreeMap::new();
        kwargs.insert(
            RencodeValue::Str(String::from("client_version")),
            RencodeValue::Str(String::from("deluge-retain/0.1.0")),
        );
        let args = vec![
            RencodeValue::Str(String::from("localclient")),
            RencodeValue::Str(String::from("secret")),
        ];

        let request = build_request(1, "daemon.login", args, kwargs);

        let outer = match &request {
            RencodeValue::List(items) if items.len() == 1 => &items[0],
            _ => panic!("expected 1-element outer list"),
        };
        let parts = match outer {
            RencodeValue::List(p) => p,
            _ => panic!("expected inner list"),
        };
        assert_eq!(parts.len(), 4, "request tuple has 4 elements");
        assert_eq!(parts[0], RencodeValue::Int(1));
        assert_eq!(parts[1], RencodeValue::Str(String::from("daemon.login")));
        match &parts[2] {
            RencodeValue::List(args_inner) => {
                assert_eq!(args_inner.len(), 2);
                assert_eq!(args_inner[0], RencodeValue::Str(String::from("localclient")));
                assert_eq!(args_inner[1], RencodeValue::Str(String::from("secret")));
            }
            _ => panic!("args is not a list"),
        }
        match &parts[3] {
            RencodeValue::Dict(map) => {
                assert_eq!(map.len(), 1);
                assert_eq!(
                    map.get(&RencodeValue::Str(String::from("client_version"))),
                    Some(&RencodeValue::Str(String::from("deluge-retain/0.1.0")))
                );
            }
            _ => panic!("kwargs is not a dict"),
        }
    }

    #[test]
    fn when_get_free_space_request_built_then_args_has_none() {
        let request = build_request(
            2,
            "core.get_free_space",
            vec![RencodeValue::None],
            BTreeMap::new(),
        );
        let parts = unwrap_request(&request);
        assert_eq!(parts[0], RencodeValue::Int(2));
        match &parts[2] {
            RencodeValue::List(args) => {
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], RencodeValue::None);
            }
            _ => panic!("args is not a list"),
        }
        match &parts[3] {
            RencodeValue::Dict(map) => assert!(map.is_empty()),
            _ => panic!("kwargs is not a dict"),
        }
    }

    #[test]
    fn when_get_torrents_status_request_built_then_filter_dict_is_first() {
        let keys = vec![RencodeValue::Str(String::from("name"))];
        let args = vec![
            RencodeValue::Dict(BTreeMap::new()),
            RencodeValue::List(keys),
        ];
        let request = build_request(3, "core.get_torrents_status", args, BTreeMap::new());
        let parts = unwrap_request(&request);
        match &parts[2] {
            RencodeValue::List(args) => {
                assert_eq!(args.len(), 2);
                assert!(
                    matches!(&args[0], RencodeValue::Dict(d) if d.is_empty()),
                    "filter_dict must be FIRST and empty"
                );
                assert!(
                    matches!(&args[1], RencodeValue::List(_)),
                    "keys must be SECOND"
                );
            }
            _ => panic!("args is not a list"),
        }
    }

    #[test]
    fn when_remove_torrent_request_built_then_args_has_id_and_true() {
        let args = vec![
            RencodeValue::Str(String::from("deadbeef")),
            RencodeValue::Bool(true),
        ];
        let request = build_request(4, "core.remove_torrent", args, BTreeMap::new());
        let parts = unwrap_request(&request);
        match &parts[2] {
            RencodeValue::List(args) => {
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], RencodeValue::Str(String::from("deadbeef")));
                assert_eq!(args[1], RencodeValue::Bool(true));
            }
            _ => panic!("args is not a list"),
        }
    }

    // ----- Response parsing unit tests -----

    #[test]
    fn when_response_is_success_then_return_value_is_extracted() {
        let response = RencodeValue::List(vec![RencodeValue::Int(1_073_741_824)]);

        let bytes = extract_single_int(&response, "core.get_free_space").expect("extract");
        assert_eq!(bytes, 1_073_741_824);
    }

    #[test]
    fn when_response_is_error_then_message_is_extractable() {
        let response = RencodeValue::List(vec![
            RencodeValue::Int(RPC_ERROR),
            RencodeValue::Str(String::from("BadLoginError")),
            RencodeValue::Str(String::from("bad password")),
            RencodeValue::Str(String::from("traceback here")),
        ]);

        let parts = response_list(&response);
        let exc_type = field_as_str(parts.get(1)).unwrap();
        let exc_msg = field_as_str(parts.get(2)).unwrap();
        assert_eq!(exc_type, "BadLoginError");
        assert_eq!(exc_msg, "bad password");
    }

    #[test]
    fn when_response_is_event_then_type_tag_is_3() {
        let response = RencodeValue::List(vec![
            RencodeValue::Int(RPC_EVENT),
            RencodeValue::Str(String::from("TorrentAddedEvent")),
            RencodeValue::List(vec![]),
        ]);
        let parts = response_list(&response);
        assert_eq!(parts[0], RencodeValue::Int(RPC_EVENT));
    }

    #[test]
    fn when_get_torrents_response_then_dict_is_parsed_into_vec() {
        let mut torrent_one = BTreeMap::new();
        torrent_one.insert(
            RencodeValue::Str(String::from("name")),
            RencodeValue::Str(String::from("torrent-one")),
        );
        torrent_one.insert(
            RencodeValue::Str(String::from("state")),
            RencodeValue::Str(String::from("Seeding")),
        );
        torrent_one.insert(
            RencodeValue::Str(String::from("progress")),
            RencodeValue::Float(100.0),
        );
        torrent_one.insert(
            RencodeValue::Str(String::from("ratio")),
            RencodeValue::Float(2.5),
        );
        torrent_one.insert(
            RencodeValue::Str(String::from("total_seeds")),
            RencodeValue::Int(10),
        );
        torrent_one.insert(
            RencodeValue::Str(String::from("num_seeds")),
            RencodeValue::Int(5),
        );
        torrent_one.insert(
            RencodeValue::Str(String::from("time_added")),
            RencodeValue::Int(1_700_000_000),
        );
        torrent_one.insert(
            RencodeValue::Str(String::from("total_done")),
            RencodeValue::Int(1_048_576),
        );
        torrent_one.insert(
            RencodeValue::Str(String::from("total_uploaded")),
            RencodeValue::Int(2_097_152),
        );
        torrent_one.insert(
            RencodeValue::Str(String::from("is_finished")),
            RencodeValue::Bool(true),
        );
        torrent_one.insert(
            RencodeValue::Str(String::from("download_location")),
            RencodeValue::Str(String::from("/data")),
        );

        let mut result_dict = BTreeMap::new();
        result_dict.insert(
            RencodeValue::Str(String::from("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111")),
            RencodeValue::Dict(torrent_one),
        );

        let response = RencodeValue::List(vec![RencodeValue::Dict(result_dict)]);

        let map = extract_single_dict(&response, "core.get_torrents_status").expect("extract");
        assert_eq!(map.len(), 1);

        let entry = map
            .get(&RencodeValue::Str(String::from(
                "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111",
            )))
            .expect("entry present");
        let fields = match entry {
            RencodeValue::Dict(f) => f,
            _ => panic!("entry is not a dict"),
        };
        let info = parse_torrent(
            "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111",
            fields,
        )
        .expect("parse");
        assert_eq!(info.name, "torrent-one");
        assert_eq!(info.state, "Seeding");
        assert!((info.progress - 100.0).abs() < f64::EPSILON);
        assert!((info.ratio - 2.5).abs() < f64::EPSILON);
        assert_eq!(info.total_seeds, 10);
        assert_eq!(info.num_seeds, 5);
        assert_eq!(info.time_added, 1_700_000_000);
        assert_eq!(info.total_done, 1_048_576);
        assert_eq!(info.total_uploaded, 2_097_152);
        assert!(info.is_finished);
        assert_eq!(info.download_location, "/data");
    }

    #[test]
    fn when_torrents_unsorted_then_parse_sorts_by_info_hash() {
        let mut fields_a = BTreeMap::new();
        fields_a.insert(
            RencodeValue::Str(String::from("name")),
            RencodeValue::Str(String::from("a")),
        );
        fields_a.insert(
            RencodeValue::Str(String::from("state")),
            RencodeValue::Str(String::from("Seeding")),
        );
        fields_a.insert(
            RencodeValue::Str(String::from("progress")),
            RencodeValue::Float(100.0),
        );
        fields_a.insert(
            RencodeValue::Str(String::from("ratio")),
            RencodeValue::Float(1.0),
        );
        fields_a.insert(
            RencodeValue::Str(String::from("total_seeds")),
            RencodeValue::Int(1),
        );
        fields_a.insert(
            RencodeValue::Str(String::from("num_seeds")),
            RencodeValue::Int(1),
        );
        fields_a.insert(
            RencodeValue::Str(String::from("time_added")),
            RencodeValue::Int(1),
        );
        fields_a.insert(
            RencodeValue::Str(String::from("total_done")),
            RencodeValue::Int(1),
        );
        fields_a.insert(
            RencodeValue::Str(String::from("total_uploaded")),
            RencodeValue::Int(1),
        );
        fields_a.insert(
            RencodeValue::Str(String::from("is_finished")),
            RencodeValue::Bool(true),
        );
        fields_a.insert(
            RencodeValue::Str(String::from("download_location")),
            RencodeValue::Str(String::from("/data")),
        );

        let fields_b = fields_a.clone();

        let mut result_dict = BTreeMap::new();
        result_dict.insert(
            RencodeValue::Str(String::from("zzzz")),
            RencodeValue::Dict(fields_a),
        );
        result_dict.insert(
            RencodeValue::Str(String::from("aaaa")),
            RencodeValue::Dict(fields_b),
        );

        let mut entries: Vec<String> = result_dict
            .iter()
            .filter_map(|(k, _)| match k {
                RencodeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        entries.sort();

        assert_eq!(entries, vec!["aaaa", "zzzz"]);
    }

    #[test]
    fn when_remove_torrent_response_then_bool_is_extracted() {
        let response = RencodeValue::List(vec![RencodeValue::Bool(true)]);
        let value = extract_single(&response, "core.remove_torrent").expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[test]
    fn when_free_space_negative_then_get_free_space_returns_error() {
        let response = RencodeValue::List(vec![RencodeValue::Int(-1)]);
        let result = extract_single_int(&response, "core.get_free_space");
        let bytes = result.expect("extract succeeds");
        assert!(bytes < 0);
    }

    #[test]
    fn when_total_seeds_out_of_u32_range_then_parse_returns_error() {
        let mut fields = BTreeMap::new();
        fields.insert(
            RencodeValue::Str(String::from("name")),
            RencodeValue::Str(String::from("x")),
        );
        fields.insert(
            RencodeValue::Str(String::from("state")),
            RencodeValue::Str(String::from("Seeding")),
        );
        fields.insert(
            RencodeValue::Str(String::from("progress")),
            RencodeValue::Float(100.0),
        );
        fields.insert(
            RencodeValue::Str(String::from("ratio")),
            RencodeValue::Float(1.0),
        );
        fields.insert(
            RencodeValue::Str(String::from("total_seeds")),
            RencodeValue::Int(i64::from(u32::MAX) + 1),
        );
        fields.insert(
            RencodeValue::Str(String::from("num_seeds")),
            RencodeValue::Int(1),
        );
        fields.insert(
            RencodeValue::Str(String::from("time_added")),
            RencodeValue::Int(1),
        );
        fields.insert(
            RencodeValue::Str(String::from("total_done")),
            RencodeValue::Int(1),
        );
        fields.insert(
            RencodeValue::Str(String::from("total_uploaded")),
            RencodeValue::Int(1),
        );
        fields.insert(
            RencodeValue::Str(String::from("is_finished")),
            RencodeValue::Bool(true),
        );
        fields.insert(
            RencodeValue::Str(String::from("download_location")),
            RencodeValue::Str(String::from("/data")),
        );

        let result = parse_torrent("deadbeef", &fields);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("total_seeds"), "got: {err}");
    }

    #[test]
    fn when_handle_response_with_matching_id_then_returns_value() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_RESPONSE),
            RencodeValue::Int(5),
            RencodeValue::List(vec![RencodeValue::Int(42)]),
        ]);
        let response = RencodeValue::List(vec![message]);

        let outcome = handle_response(&response, 5, "test").expect("ok");
        match outcome {
            ResponseOutcome::Return(RencodeValue::List(items)) => {
                assert_eq!(items, vec![RencodeValue::Int(42)]);
            }
            other => panic!("expected Return, got {other:?}"),
        }
    }

    #[test]
    fn when_handle_response_with_mismatched_id_then_continues() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_RESPONSE),
            RencodeValue::Int(99),
            RencodeValue::List(vec![RencodeValue::Int(42)]),
        ]);
        let response = RencodeValue::List(vec![message]);

        let outcome = handle_response(&response, 5, "test").expect("ok");
        assert!(matches!(outcome, ResponseOutcome::Continue));
    }

    #[test]
    fn when_handle_response_with_event_then_continues() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_EVENT),
            RencodeValue::Str(String::from("TorrentAddedEvent")),
            RencodeValue::List(vec![]),
        ]);
        let response = RencodeValue::List(vec![message]);

        let outcome = handle_response(&response, 5, "test").expect("ok");
        assert!(matches!(outcome, ResponseOutcome::Continue));
    }

    #[test]
    fn when_handle_response_with_error_then_returns_error() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_ERROR),
            RencodeValue::Str(String::from("BadLoginError")),
            RencodeValue::Str(String::from("bad password")),
            RencodeValue::Str(String::from("tb")),
        ]);
        let response = RencodeValue::List(vec![message]);

        let err = handle_response(&response, 5, "test").expect_err("should error");
        let msg = err.to_string();
        assert!(msg.contains("BadLoginError"), "got: {msg}");
        assert!(msg.contains("bad password"), "got: {msg}");
    }

    // ----- helpers -----

    fn unwrap_request(value: &RencodeValue) -> Vec<RencodeValue> {
        let outer = match value {
            RencodeValue::List(items) if items.len() == 1 => &items[0],
            _ => panic!("expected 1-element outer list"),
        };
        match outer {
            RencodeValue::List(p) => p.clone(),
            _ => panic!("expected inner list"),
        }
    }

    fn response_list(value: &RencodeValue) -> Vec<RencodeValue> {
        match value {
            RencodeValue::List(p) => p.clone(),
            _ => panic!("expected list"),
        }
    }
}