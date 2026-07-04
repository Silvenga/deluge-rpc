//! Mock Deluge daemon for integration and e2e tests.
//!
//! Spawns a TCP+TLS server speaking the Deluge daemon RPC wire protocol
//! (5-byte header + zlib + rencode) on a random localhost port. The mock
//! accepts a single connection, decodes each framed request, records the
//! method name, and replies with a pre-configured canned response.
//!
//! Only the four methods used by [`crate::client::DelugeClient`] are
//! supported: `daemon.login`, `core.get_free_space`,
//! `core.get_torrents_status`, and `core.remove_torrent`. Any other
//! method receives an `RPC_ERROR` reply.

#![cfg(test)]

use std::collections::BTreeMap;
use std::io::{self, ErrorKind, Read, Write as _};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, Once};

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use rustls::crypto;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::ServerConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

use crate::rencode::{decode, encode, RencodeValue};

/// Wire protocol version (Deluge 2.x) — matches [`crate::transport`].
const PROTOCOL_VERSION: u8 = 1;
/// Fixed header size: 1 byte version + 4 bytes big-endian body length.
const HEADER_LEN: usize = 5;

/// RPC response message type tag.
const RPC_RESPONSE: i64 = 1;
/// RPC error message type tag.
const RPC_ERROR: i64 = 2;

/// Install the `ring` crypto provider as the process default once.
///
/// Mirrors the guard in [`crate::transport`]; the `Once` is local to this
/// module so the two do not race — `install_default` is idempotent.
fn ensure_crypto_provider() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let _ = crypto::ring::default_provider().install_default();
    });
}

/// Canned response for a single RPC method.
#[derive(Debug, Clone, Default)]
pub enum MockResponse {
    /// Respond with `[1, id, [value]]` — a successful RPC response whose
    /// return list contains `value`.
    Success(RencodeValue),
    /// Respond with `[2, id, exc_type, exc_msg, ""]` — an RPC error.
    Error {
        exc_type: String,
        exc_msg: String,
    },
    /// No canned response configured; reply with a generic RPC error.
    #[default]
    NotConfigured,
}

impl MockResponse {
    fn success(value: RencodeValue) -> Self {
        Self::Success(value)
    }

    fn error(exc_type: &str, exc_msg: &str) -> Self {
        Self::Error {
            exc_type: String::from(exc_type),
            exc_msg: String::from(exc_msg),
        }
    }
}

/// Configuration for a [`MockDelugeDaemon`]: one canned response per
/// supported method.
#[derive(Debug, Clone, Default)]
pub struct MockDaemonConfig {
    /// Canned response for `daemon.login`.
    pub login: MockResponse,
    /// Canned response for `core.get_free_space`.
    pub free_space: MockResponse,
    /// Canned response for `core.get_torrents_status`.
    pub torrents: MockResponse,
    /// Canned response for `core.remove_torrent`.
    pub remove: MockResponse,
}

impl MockDaemonConfig {
    /// Login succeeds with auth level `5` (the default Deluge admin level).
    pub fn login_ok() -> Self {
        Self {
            login: MockResponse::success(RencodeValue::Int(5)),
            ..Self::default()
        }
    }

    /// Login fails with a `BadLoginError`.
    pub fn login_bad() -> Self {
        Self {
            login: MockResponse::error("BadLoginError", "bad password"),
            ..Self::default()
        }
    }
}

/// A running mock Deluge daemon.
///
/// Spawned by [`MockDelugeDaemon::start`]; the server task is tracked via
/// a [`JoinHandle`] and the bound address is exposed for connecting a
/// [`crate::client::DelugeClient`]. Method names received from the client
/// are recorded in a shared vector for test assertions.
pub struct MockDelugeDaemon {
    addr: SocketAddr,
    handle: JoinHandle<()>,
    received: Arc<Mutex<Vec<String>>>,
}

impl MockDelugeDaemon {
    /// Start a mock daemon bound to `127.0.0.1:0` with the given config.
    ///
    /// The server accepts a single TLS connection and serves the canned
    /// responses in `config`, recording each received method name.
    #[expect(
        clippy::expect_used,
        reason = "test helper: bind/local_addr failures are fatal and unrecoverable"
    )]
    pub async fn start(config: MockDaemonConfig) -> Self {
        ensure_crypto_provider();
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("mock daemon: bind 127.0.0.1:0");
        let addr = listener.local_addr().expect("mock daemon: local_addr");
        let server_config = self_signed_server_config();
        let acceptor = TlsAcceptor::from(server_config);
        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let received_clone = Arc::clone(&received);
        let handle = tokio::spawn(async move {
            // Best-effort server: errors are swallowed since the test
            // inspects the client-side result and the received-methods
            // vector.
            let Ok((tcp, _)) = listener.accept().await else {
                return;
            };
            let Ok(mut tls) = acceptor.accept(tcp).await else {
                return;
            };
            let _ = serve(&mut tls, &config, &received_clone).await;
        });

        Self {
            addr,
            handle,
            received,
        }
    }

    /// The bound IP address as a string (suitable for
    /// [`crate::client::DelugeClient::new`]).
    pub fn host(&self) -> String {
        self.addr.ip().to_string()
    }

    /// The bound TCP port.
    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// Method names received from the client, in arrival order.
    #[expect(
        clippy::expect_used,
        reason = "test helper: a poisoned received-methods mutex is a fatal test bug"
    )]
    pub fn received_methods(&self) -> Vec<String> {
        self.received
            .lock()
            .expect("received-methods mutex poisoned")
            .clone()
    }
}

impl Drop for MockDelugeDaemon {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Build a self-signed cert + matching rustls server config.
///
/// Mirrors the test helper in [`crate::transport`]; duplicated here so the
/// mock daemon stays self-contained and does not depend on private test
/// items in another module.
#[expect(
    clippy::expect_used,
    reason = "test helper: rcgen/server-config failures are fatal and unrecoverable"
)]
fn self_signed_server_config() -> Arc<ServerConfig> {
    ensure_crypto_provider();
    let key_pair = rcgen::KeyPair::generate().expect("mock daemon: generate key pair");
    let cert_params = rcgen::CertificateParams::new(vec!["localhost".to_owned()])
        .expect("mock daemon: cert params");
    let cert = cert_params
        .self_signed(&key_pair)
        .expect("mock daemon: self-signed cert");
    let cert_der = CertificateDer::from(cert.der().to_vec());
    let key_der: PrivateKeyDer =
        PrivatePkcs8KeyDer::from(key_pair.serialize_der()).into();
    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .expect("mock daemon: server config");
    Arc::new(server_config)
}

/// Serve one TLS connection: read framed requests, record method names,
/// and write canned responses.
#[expect(
    clippy::expect_used,
    reason = "test helper: a poisoned received-methods mutex is a fatal test bug"
)]
async fn serve(
    tls: &mut TlsStream<TcpStream>,
    config: &MockDaemonConfig,
    received: &Mutex<Vec<String>>,
) -> io::Result<()> {
    loop {
        let raw = match read_frame(tls).await {
            Ok(bytes) => bytes,
            Err(ReadFrameError::Eof) => return Ok(()),
            Err(ReadFrameError::Io(e)) => return Err(e),
        };
        let Ok(decoded) = decode(&raw) else {
            continue;
        };
        let Some((id, method)) = extract_request(&decoded) else {
            continue;
        };
        {
            let mut guard = received.lock().expect("received mutex poisoned");
            guard.push(method.clone());
        }
        let response = build_response(id, &method, config);
        let encoded = encode(&response);
        write_frame(tls, &encoded).await?;
    }
}

/// Read one framed payload (5-byte header + zlib body, decompressed).
async fn read_frame(tls: &mut TlsStream<TcpStream>) -> Result<Vec<u8>, ReadFrameError> {
    let mut header = [0u8; HEADER_LEN];
    match tls.read_exact(&mut header).await {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
            return Err(ReadFrameError::Eof);
        }
        Err(e) => return Err(ReadFrameError::Io(e)),
    }
    if header[0] != PROTOCOL_VERSION {
        return Err(ReadFrameError::Io(io::Error::new(
            ErrorKind::InvalidData,
            "protocol version mismatch",
        )));
    }
    let body_len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
    let mut body = vec![0u8; usize::try_from(body_len).map_err(|_| {
        ReadFrameError::Io(io::Error::new(
            ErrorKind::InvalidData,
            "body length overflow",
        ))
    })?];
    tls.read_exact(&mut body)
        .await
        .map_err(ReadFrameError::Io)?;
    zlib_decompress(&body)
        .map_err(|e| ReadFrameError::Io(io::Error::new(ErrorKind::InvalidData, e)))
}

#[derive(Debug)]
enum ReadFrameError {
    Eof,
    Io(io::Error),
}

/// Write one framed payload (zlib-compress + 5-byte header).
async fn write_frame(tls: &mut TlsStream<TcpStream>, data: &[u8]) -> io::Result<()> {
    let compressed = zlib_compress(data)?;
    let len = u32::try_from(compressed.len()).map_err(|_| {
        io::Error::new(ErrorKind::InvalidInput, "payload too large for u32 len")
    })?;
    let mut frame = Vec::with_capacity(HEADER_LEN + compressed.len());
    frame.push(PROTOCOL_VERSION);
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(&compressed);
    tls.write_all(&frame).await?;
    tls.flush().await?;
    Ok(())
}

/// Extract `(request_id, method_name)` from a decoded request envelope
/// `[[id, method, [args], {kwargs}]]`.
#[expect(
    clippy::indexing_slicing,
    reason = "request shape is validated by guards; parts[0]/parts[1] are safe after len check"
)]
fn extract_request(decoded: &RencodeValue) -> Option<(i64, String)> {
    let outer = match decoded {
        RencodeValue::List(items) if items.len() == 1 => &items[0],
        _ => return None,
    };
    let RencodeValue::List(parts) = outer else {
        return None;
    };
    if parts.len() < 2 {
        return None;
    }
    let id = match &parts[0] {
        RencodeValue::Int(i) => *i,
        _ => return None,
    };
    let method = match &parts[1] {
        RencodeValue::Str(s) => s.clone(),
        _ => return None,
    };
    Some((id, method))
}

/// Build the response envelope for a given request id + method, using the
/// canned response from `config`.
fn build_response(id: i64, method: &str, config: &MockDaemonConfig) -> RencodeValue {
    let canned = match method {
        "daemon.login" => &config.login,
        "core.get_free_space" => &config.free_space,
        "core.get_torrents_status" => &config.torrents,
        "core.remove_torrent" => &config.remove,
        _ => &MockResponse::NotConfigured,
    };
    let inner = match canned {
        MockResponse::Success(value) => vec![
            RencodeValue::Int(RPC_RESPONSE),
            RencodeValue::Int(id),
            RencodeValue::List(vec![value.clone()]),
        ],
        MockResponse::Error { exc_type, exc_msg } => vec![
            RencodeValue::Int(RPC_ERROR),
            RencodeValue::Int(id),
            RencodeValue::Str(exc_type.clone()),
            RencodeValue::Str(exc_msg.clone()),
            RencodeValue::Str(String::new()),
        ],
        MockResponse::NotConfigured => vec![
            RencodeValue::Int(RPC_ERROR),
            RencodeValue::Int(id),
            RencodeValue::Str(String::from("NotConfigured")),
            RencodeValue::Str(format!("mock has no canned response for `{method}`")),
            RencodeValue::Str(String::new()),
        ],
    };
    // The daemon wraps the message in a 1-element outer list.
    RencodeValue::List(vec![RencodeValue::List(inner)])
}

/// zlib-compress `data` (zlib wrapper, not gzip/deflate).
fn zlib_compress(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// zlib-decompress `data`.
fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    match decoder.read_to_end(&mut out) {
        Ok(_) => Ok(out),
        Err(e) => Err(e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "mock daemon tests use expect for clarity on shape errors"
)]
#[expect(
    clippy::unwrap_used,
    reason = "mock daemon tests use unwrap for clarity on known-good values"
)]
#[expect(
    clippy::indexing_slicing,
    reason = "mock daemon tests index known-length vectors"
)]
mod tests {
    use super::*;
    use crate::client::{DelugeClient, DelugeRpc};
    use crate::torrent::TorrentInfo;

    use std::string::ToString;

    const GB: u64 = 1_073_741_824;

    /// Build a single-torrent dict suitable for `core.get_torrents_status`.
    fn torrents_dict(info_hash: &str, name: &str) -> RencodeValue {
        let mut fields = BTreeMap::new();
        fields.insert(
            RencodeValue::Str(String::from("name")),
            RencodeValue::Str(String::from(name)),
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
            RencodeValue::Float(2.5),
        );
        fields.insert(
            RencodeValue::Str(String::from("total_seeds")),
            RencodeValue::Int(10),
        );
        fields.insert(
            RencodeValue::Str(String::from("num_seeds")),
            RencodeValue::Int(5),
        );
        fields.insert(
            RencodeValue::Str(String::from("time_added")),
            RencodeValue::Int(1_700_000_000),
        );
        fields.insert(
            RencodeValue::Str(String::from("total_done")),
            RencodeValue::Int(i64::try_from(2 * GB).unwrap()),
        );
        fields.insert(
            RencodeValue::Str(String::from("total_uploaded")),
            RencodeValue::Int(i64::try_from(4 * GB).unwrap()),
        );
        fields.insert(
            RencodeValue::Str(String::from("is_finished")),
            RencodeValue::Bool(true),
        );
        fields.insert(
            RencodeValue::Str(String::from("download_location")),
            RencodeValue::Str(String::from("/data")),
        );

        let mut dict = BTreeMap::new();
        dict.insert(
            RencodeValue::Str(String::from(info_hash)),
            RencodeValue::Dict(fields),
        );
        RencodeValue::Dict(dict)
    }

    #[tokio::test]
    async fn when_login_and_get_free_space_then_roundtrip_succeeds() {
        let config = MockDaemonConfig {
            login: MockResponse::success(RencodeValue::Int(5)),
            free_space: MockResponse::success(RencodeValue::Int(i64::try_from(GB).unwrap())),
            ..MockDaemonConfig::default()
        };
        let mock = MockDelugeDaemon::start(config).await;

        let client = DelugeClient::new(
            mock.host(),
            mock.port(),
            String::from("localclient"),
            String::from("password"),
        );

        client
            .login()
            .await
            .expect("login against mock daemon succeeds");

        let free = client
            .get_free_space()
            .await
            .expect("get_free_space against mock daemon succeeds");
        assert_eq!(free, GB);

        let methods = mock.received_methods();
        assert!(
            methods.iter().any(|m| m == "daemon.login"),
            "received_methods should contain daemon.login, got {methods:?}"
        );
        assert!(
            methods
                .iter()
                .any(|m| m == "core.get_free_space"),
            "received_methods should contain core.get_free_space, got {methods:?}"
        );
    }

    #[tokio::test]
    async fn when_login_fails_then_client_returns_error() {
        let mock = MockDelugeDaemon::start(MockDaemonConfig::login_bad()).await;

        let client = DelugeClient::new(
            mock.host(),
            mock.port(),
            String::from("localclient"),
            String::from("wrong"),
        );

        let result = client.login().await;
        assert!(
            result.is_err(),
            "login with a BadLoginError response must fail"
        );
        let chain = result
            .unwrap_err()
            .chain()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" -> ");
        assert!(chain.contains("BadLoginError"), "got: {chain}");
    }

    #[tokio::test]
    async fn when_get_torrents_then_mock_returns_configured_dict() {
        let info_hash = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111";
        let config = MockDaemonConfig {
            login: MockResponse::success(RencodeValue::Int(5)),
            torrents: MockResponse::success(torrents_dict(info_hash, "torrent-one")),
            ..MockDaemonConfig::default()
        };
        let mock = MockDelugeDaemon::start(config).await;

        let client = DelugeClient::new(
            mock.host(),
            mock.port(),
            String::from("localclient"),
            String::from("password"),
        );
        client.login().await.expect("login");

        let torrents: Vec<TorrentInfo> = client
            .get_torrents()
            .await
            .expect("get_torrents against mock daemon succeeds");
        assert_eq!(torrents.len(), 1);
        assert_eq!(torrents[0].info_hash, info_hash);
        assert_eq!(torrents[0].name, "torrent-one");
        assert_eq!(torrents[0].total_done, 2 * GB);
    }

    #[tokio::test]
    async fn when_remove_torrent_then_mock_returns_true_and_records_method() {
        let info_hash = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
        let config = MockDaemonConfig {
            login: MockResponse::success(RencodeValue::Int(5)),
            remove: MockResponse::success(RencodeValue::Bool(true)),
            ..MockDaemonConfig::default()
        };
        let mock = MockDelugeDaemon::start(config).await;

        let client = DelugeClient::new(
            mock.host(),
            mock.port(),
            String::from("localclient"),
            String::from("password"),
        );
        client.login().await.expect("login");

        let removed = client
            .remove_torrent(info_hash)
            .await
            .expect("remove_torrent against mock daemon succeeds");
        assert!(removed, "mock should report removal succeeded");

        let methods = mock.received_methods();
        assert!(
            methods.iter().any(|m| m == "core.remove_torrent"),
            "received_methods should contain core.remove_torrent, got {methods:?}"
        );
    }

    #[tokio::test]
    async fn when_method_not_configured_then_client_receives_error() {
        // Login configured, but free_space left as NotConfigured.
        let mock = MockDelugeDaemon::start(MockDaemonConfig::login_ok()).await;

        let client = DelugeClient::new(
            mock.host(),
            mock.port(),
            String::from("localclient"),
            String::from("password"),
        );
        client.login().await.expect("login");

        let result = client.get_free_space().await;
        assert!(result.is_err(), "NotConfigured method must error");
        let chain = result
            .unwrap_err()
            .chain()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" -> ");
        assert!(chain.contains("NotConfigured"), "got: {chain}");
    }
}