use crate::Matcher;
use crate::{Interaction, Response as CassetteResponse};
use deluge_rpc::RencodeValue;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use rustls::crypto;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::ServerConfig;
use std::io::{self, ErrorKind, Read, Write};
use std::net::SocketAddr;
use std::sync::{Arc, Once};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;

const PROTOCOL_VERSION: u8 = 1;
const HEADER_LEN: usize = 5;
const RPC_RESPONSE: i64 = 1;
const RPC_ERROR: i64 = 2;

#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),
}

fn ensure_crypto_provider() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let _ = crypto::ring::default_provider().install_default();
    });
}

fn self_signed_server_config() -> Arc<ServerConfig> {
    ensure_crypto_provider();
    let key_pair = rcgen::KeyPair::generate().expect("generate key pair");
    let cert_params =
        rcgen::CertificateParams::new(vec!["localhost".to_owned()]).expect("cert params");
    let cert = cert_params
        .self_signed(&key_pair)
        .expect("self-signed cert");
    let cert_der = CertificateDer::from(cert.der().to_vec());
    let key_der: PrivateKeyDer = PrivatePkcs8KeyDer::from(key_pair.serialize_der()).into();
    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .expect("server config");
    Arc::new(server_config)
}

pub struct ReplayServer {
    addr: SocketAddr,
    handle: JoinHandle<()>,
    matcher: Arc<Matcher>,
}

impl ReplayServer {
    pub async fn start(matcher: Arc<Matcher>) -> Result<Self, ReplayError> {
        ensure_crypto_provider();
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let server_config = self_signed_server_config();
        let acceptor = TlsAcceptor::from(server_config);
        let matcher_clone = Arc::clone(&matcher);

        let handle = tokio::spawn(async move {
            loop {
                let (tcp, _) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(_) => break,
                };
                let tls = match acceptor.accept(tcp).await {
                    Ok(tls) => tls,
                    Err(_) => continue,
                };
                let m = Arc::clone(&matcher_clone);
                tokio::spawn(async move {
                    serve_connection(tls, &m).await;
                });
            }
        });

        Ok(ReplayServer {
            addr,
            handle,
            matcher,
        })
    }

    pub fn host(&self) -> String {
        self.addr.ip().to_string()
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn consumed_methods(&self) -> Vec<String> {
        self.matcher.consumed_methods()
    }
}

impl Drop for ReplayServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn serve_connection(mut tls: TlsStream<TcpStream>, matcher: &Matcher) {
    loop {
        let raw = match read_frame(&mut tls).await {
            Ok(bytes) => bytes,
            Err(ReadFrameError::Eof) => return,
            Err(ReadFrameError::Io(e)) => {
                tracing::debug!(error = %e, "read frame error, closing connection");
                return;
            }
        };
        let decoded = match RencodeValue::decode(&raw) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let Some((id, method, args)) = extract_request(&decoded) else {
            continue;
        };
        let response = match matcher.find_match(&method, &args) {
            Some(interaction) => build_response_from_interaction(id, &interaction),
            None => build_unknown_method_response(id, &method),
        };
        let encoded = response.encode();
        if write_frame(&mut tls, &encoded).await.is_err() {
            return;
        }
    }
}

enum ReadFrameError {
    Eof,
    Io(io::Error),
}

async fn read_frame(tls: &mut TlsStream<TcpStream>) -> Result<Vec<u8>, ReadFrameError> {
    let mut header = [0u8; HEADER_LEN];
    match tls.read_exact(&mut header).await {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Err(ReadFrameError::Eof),
        Err(e) => return Err(ReadFrameError::Io(e)),
    }
    if header[0] != PROTOCOL_VERSION {
        return Err(ReadFrameError::Io(io::Error::new(
            ErrorKind::InvalidData,
            "protocol version mismatch",
        )));
    }
    let body_len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
    let mut body = vec![
        0u8;
        usize::try_from(body_len).map_err(|_| {
            ReadFrameError::Io(io::Error::new(
                ErrorKind::InvalidData,
                "body length overflow",
            ))
        })?
    ];
    match tls.read_exact(&mut body).await {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Err(ReadFrameError::Eof),
        Err(e) => return Err(ReadFrameError::Io(e)),
    }
    zlib_decompress(&body)
        .map_err(|e| ReadFrameError::Io(io::Error::new(ErrorKind::InvalidData, e)))
}

async fn write_frame(tls: &mut TlsStream<TcpStream>, data: &[u8]) -> io::Result<()> {
    let compressed = zlib_compress(data)?;
    let len = u32::try_from(compressed.len())
        .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "payload too large for u32 len"))?;
    let mut frame = Vec::with_capacity(HEADER_LEN + compressed.len());
    frame.push(PROTOCOL_VERSION);
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(&compressed);
    tls.write_all(&frame).await?;
    tls.flush().await?;
    Ok(())
}

fn zlib_compress(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

fn extract_request(decoded: &RencodeValue) -> Option<(u32, String, RencodeValue)> {
    let outer = match decoded {
        RencodeValue::List(items) if items.len() == 1 => &items[0],
        _ => return None,
    };
    let inner = match outer {
        RencodeValue::List(parts) if parts.len() >= 2 => parts,
        _ => return None,
    };
    let id = match &inner[0] {
        RencodeValue::Int(i) => u32::try_from(*i).ok()?,
        _ => return None,
    };
    let method = match &inner[1] {
        RencodeValue::Str(s) => s.clone(),
        _ => return None,
    };
    let args = inner.get(2).cloned().unwrap_or(RencodeValue::List(vec![]));
    Some((id, method, args))
}

fn build_response_from_interaction(id: u32, interaction: &Interaction) -> RencodeValue {
    let inner = match &interaction.response {
        CassetteResponse::Ok { value } => vec![
            RencodeValue::Int(RPC_RESPONSE),
            RencodeValue::Int(i64::from(id)),
            RencodeValue::List(vec![value.clone()]),
        ],
        CassetteResponse::Error {
            exc_type,
            exc_msg,
            traceback,
        } => vec![
            RencodeValue::Int(RPC_ERROR),
            RencodeValue::Int(i64::from(id)),
            RencodeValue::Str(exc_type.clone()),
            RencodeValue::Str(exc_msg.clone()),
            RencodeValue::Str(traceback.clone()),
        ],
    };
    RencodeValue::List(vec![RencodeValue::List(inner)])
}

fn build_unknown_method_response(id: u32, method: &str) -> RencodeValue {
    let inner = vec![
        RencodeValue::Int(RPC_ERROR),
        RencodeValue::Int(i64::from(id)),
        RencodeValue::Str("UnknownMethod".into()),
        RencodeValue::Str(format!("no cassette entry for method '{method}'")),
        RencodeValue::Str(String::new()),
    ];
    RencodeValue::List(vec![RencodeValue::List(inner)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cassette, Matcher, Request, Response as CassetteResponse};
    use deluge_rpc::DaemonRpc;
    use deluge_rpc::DelugeClient;
    use deluge_rpc::RencodeValue;

    fn make_cassette(interactions: Vec<Interaction>) -> Cassette {
        Cassette {
            version: 1,
            recorded_at: "2026-07-04T12:00:00Z".into(),
            daemon_version: None,
            interactions,
        }
    }

    fn login_interaction() -> Interaction {
        Interaction {
            request: Request {
                method: "daemon.login".into(),
                args: RencodeValue::List(vec![
                    RencodeValue::Str("testuser".into()),
                    RencodeValue::Str("testpass".into()),
                ]),
                kwargs: RencodeValue::List(vec![]),
            },
            response: CassetteResponse::Ok {
                value: RencodeValue::Int(10),
            },
        }
    }

    fn ok_interaction(method: &str, args: RencodeValue, value: RencodeValue) -> Interaction {
        Interaction {
            request: Request {
                method: method.into(),
                args,
                kwargs: RencodeValue::List(vec![]),
            },
            response: CassetteResponse::Ok { value },
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_cassette_loaded_then_replay_matches_exact_args() {
        let cassette = make_cassette(vec![
            login_interaction(),
            ok_interaction(
                "daemon.info",
                RencodeValue::List(vec![RencodeValue::None]),
                RencodeValue::Str("2.1.1".into()),
            ),
        ]);
        let matcher = Arc::new(Matcher::new(cassette.interactions));
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClient::connect(&server.host(), server.port(), "testuser", "testpass")
            .await
            .expect("connect");

        let result = client.daemon().info().await;
        assert!(result.is_ok(), "daemon.info should succeed: {result:?}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_no_exact_match_then_fallback_to_unmatched_for_method() {
        let cassette = make_cassette(vec![
            login_interaction(),
            ok_interaction(
                "daemon.get_version",
                RencodeValue::List(vec![RencodeValue::Str("v1".into())]),
                RencodeValue::Str("version-one".into()),
            ),
            ok_interaction(
                "daemon.get_version",
                RencodeValue::List(vec![RencodeValue::Str("v2".into())]),
                RencodeValue::Str("version-two".into()),
            ),
        ]);
        let matcher = Arc::new(Matcher::new(cassette.interactions));
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClient::connect(&server.host(), server.port(), "testuser", "testpass")
            .await
            .expect("connect");

        let result = client.daemon().get_version().await;
        assert!(
            result.is_ok(),
            "first call (fallback) should succeed: {result:?}"
        );
        assert_eq!(result.unwrap(), "version-one");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_entry_consumed_then_not_rematched() {
        let cassette = make_cassette(vec![
            login_interaction(),
            ok_interaction(
                "daemon.info",
                RencodeValue::List(vec![RencodeValue::None]),
                RencodeValue::Str("only-one".into()),
            ),
        ]);
        let matcher = Arc::new(Matcher::new(cassette.interactions));
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClient::connect(&server.host(), server.port(), "testuser", "testpass")
            .await
            .expect("connect");

        let first = client.daemon().info().await;
        assert!(first.is_ok(), "first call should succeed: {first:?}");

        let second = client.daemon().info().await;
        assert!(
            second.is_err(),
            "second call should fail (consumed): {second:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_error_response_then_client_receives_error() {
        let cassette = make_cassette(vec![
            login_interaction(),
            Interaction {
                request: Request {
                    method: "daemon.info".into(),
                    args: RencodeValue::List(vec![RencodeValue::None]),
                    kwargs: RencodeValue::List(vec![]),
                },
                response: CassetteResponse::Error {
                    exc_type: "TestError".into(),
                    exc_msg: "this is a test error".into(),
                    traceback: String::new(),
                },
            },
        ]);
        let matcher = Arc::new(Matcher::new(cassette.interactions));
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClient::connect(&server.host(), server.port(), "testuser", "testpass")
            .await
            .expect("connect");

        let result = client.daemon().info().await;
        assert!(result.is_err(), "should receive error: {result:?}");
        let err_msg = format!("{result:?}");
        assert!(
            err_msg.contains("TestError"),
            "error should mention TestError, got: {err_msg}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_login_in_cassette_then_replay_serves_it() {
        let cassette = make_cassette(vec![login_interaction()]);
        let matcher = Arc::new(Matcher::new(cassette.interactions));
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClient::connect(&server.host(), server.port(), "testuser", "testpass")
            .await
            .expect("connect");

        let version = client.daemon().get_version().await;
        assert!(
            version.is_err(),
            "calling non-existent method should fail, got: {version:?}"
        );
        let err_msg = format!("{version:?}");
        assert!(
            err_msg.contains("UnknownMethod"),
            "error should mention UnknownMethod, got: {err_msg}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_unknown_method_then_error_or_skip() {
        let cassette = make_cassette(vec![login_interaction()]);
        let matcher = Arc::new(Matcher::new(cassette.interactions));
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClient::connect(&server.host(), server.port(), "testuser", "testpass")
            .await
            .expect("connect");

        let result = client.daemon().info().await;
        assert!(result.is_err(), "unknown method should error: {result:?}");
        let err_msg = format!("{result:?}");
        assert!(
            err_msg.contains("UnknownMethod"),
            "error should contain UnknownMethod, got: {err_msg}"
        );
    }
}
