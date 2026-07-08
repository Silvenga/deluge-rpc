//! Integration tests for the deluge-cli binary.

use assert_cmd::Command;
use assert_fs::TempDir;
use deluge_rpc_client::RencodeValue;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use rustls::crypto::ring;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::ServerConfig;
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Once;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;

const HEADER_LEN: usize = 5;
const PROTOCOL_VERSION: u8 = 1;

fn install_crypto_provider() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let _ = ring::default_provider().install_default();
    });
}

fn self_signed_server_config() -> Arc<ServerConfig> {
    install_crypto_provider();
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

fn login_response_frame(request_id: u32) -> Vec<u8> {
    let response = RencodeValue::List(vec![
        RencodeValue::Int(1),
        RencodeValue::Int(i64::from(request_id)),
        RencodeValue::Int(10),
    ]);
    let payload = response.encode();
    frame_encode(&payload)
}

fn rpc_response_frame(request_id: u32, return_value: RencodeValue) -> Vec<u8> {
    let response = RencodeValue::List(vec![
        RencodeValue::Int(1),
        RencodeValue::Int(i64::from(request_id)),
        return_value,
    ]);
    let payload = response.encode();
    frame_encode(&payload)
}

fn frame_encode(payload: &[u8]) -> Vec<u8> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    enc.write_all(payload).expect("compress");
    let compressed = enc.finish().expect("finish");
    let len = compressed.len() as u32;
    let mut frame = Vec::with_capacity(HEADER_LEN + compressed.len());
    frame.push(PROTOCOL_VERSION);
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(&compressed);
    frame
}

fn decode_frame(body: &[u8]) -> Option<RencodeValue> {
    let mut dec = ZlibDecoder::new(body);
    let mut out = Vec::new();
    let _ = dec.read_to_end(&mut out);
    RencodeValue::decode(&out).ok()
}

struct MockDaemon {
    addr: SocketAddr,
}

impl MockDaemon {
    async fn new() -> Self {
        install_crypto_provider();
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let server_config = self_signed_server_config();
        let acceptor = tokio_rustls::TlsAcceptor::from(server_config);

        tokio::spawn(async move {
            loop {
                let (tcp, _) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(_) => break,
                };
                let tls = match acceptor.accept(tcp).await {
                    Ok(tls) => tls,
                    Err(_) => continue,
                };
                tokio::spawn(async move {
                    handle_connection(tls).await;
                });
            }
        });

        MockDaemon { addr }
    }
}

async fn handle_connection(mut tls: TlsStream<TcpStream>) {
    let mut header = [0u8; HEADER_LEN];
    loop {
        if tls.read_exact(&mut header).await.is_err() {
            break;
        }
        let body_len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
        if body_len > 16 * 1024 * 1024 {
            break;
        }
        let mut body = vec![0u8; body_len as usize];
        if tls.read_exact(&mut body).await.is_err() {
            break;
        }

        let Some(decoded) = decode_frame(&body) else {
            break;
        };

        let (request_id, is_login, method) = extract_request_info(&decoded);

        if is_login {
            let response_frame = login_response_frame(request_id);
            if tls.write_all(&response_frame).await.is_err() {
                break;
            }
            let _ = tls.flush().await;
        } else {
            let return_value = match method.as_str() {
                "daemon.info" => RencodeValue::Str("2.1.1".into()),
                "daemon.get_version" => RencodeValue::Str("2.1.1".into()),
                "daemon.get_method_list" => RencodeValue::List(vec![
                    RencodeValue::Str("daemon.login".into()),
                    RencodeValue::Str("core.get_free_space".into()),
                ]),
                "core.get_free_space" => RencodeValue::Int(1073741824),
                "core.get_torrents_status" => RencodeValue::Dict(BTreeMap::new()),
                "core.get_torrent_status" => RencodeValue::Dict(BTreeMap::new()),
                "core.get_session_status" => {
                    let mut map = BTreeMap::new();
                    map.insert(
                        RencodeValue::Str("download_rate".into()),
                        RencodeValue::Float(1024.0),
                    );
                    map.insert(
                        RencodeValue::Str("upload_rate".into()),
                        RencodeValue::Float(512.0),
                    );
                    map.insert(
                        RencodeValue::Str("payload_download_rate".into()),
                        RencodeValue::Float(1000.0),
                    );
                    map.insert(
                        RencodeValue::Str("payload_upload_rate".into()),
                        RencodeValue::Float(500.0),
                    );
                    map.insert(
                        RencodeValue::Str("ip_overhead_download_rate".into()),
                        RencodeValue::Float(10.0),
                    );
                    map.insert(
                        RencodeValue::Str("ip_overhead_upload_rate".into()),
                        RencodeValue::Float(5.0),
                    );
                    map.insert(
                        RencodeValue::Str("tracker_download_rate".into()),
                        RencodeValue::Float(0.0),
                    );
                    map.insert(
                        RencodeValue::Str("tracker_upload_rate".into()),
                        RencodeValue::Float(0.0),
                    );
                    map.insert(
                        RencodeValue::Str("dht_download_rate".into()),
                        RencodeValue::Float(14.0),
                    );
                    map.insert(
                        RencodeValue::Str("dht_upload_rate".into()),
                        RencodeValue::Float(7.0),
                    );
                    map.insert(
                        RencodeValue::Str("write_hit_ratio".into()),
                        RencodeValue::Float(0.95),
                    );
                    map.insert(
                        RencodeValue::Str("read_hit_ratio".into()),
                        RencodeValue::Float(0.88),
                    );
                    RencodeValue::Dict(map)
                }
                "core.get_config_value" => RencodeValue::Str("/downloads".into()),
                "core.get_config" => {
                    let mut map = BTreeMap::new();
                    map.insert(
                        RencodeValue::Str("download_location".into()),
                        RencodeValue::Str("/downloads".into()),
                    );
                    map.insert(
                        RencodeValue::Str("max_upload_speed".into()),
                        RencodeValue::Float(-1.0),
                    );
                    RencodeValue::Dict(map)
                }
                "core.get_enabled_plugins" => {
                    RencodeValue::List(vec![RencodeValue::Str("Label".into())])
                }
                "label.get_labels" => RencodeValue::List(vec![
                    RencodeValue::Str("movies".into()),
                    RencodeValue::Str("music".into()),
                ]),
                _ => RencodeValue::Str("ok".into()),
            };
            let response_frame = rpc_response_frame(request_id, return_value);
            if tls.write_all(&response_frame).await.is_err() {
                break;
            }
            let _ = tls.flush().await;
        }
    }
}

fn extract_request_info(decoded: &RencodeValue) -> (u32, bool, String) {
    match decoded {
        RencodeValue::List(items) if items.len() == 1 => match &items[0] {
            RencodeValue::List(inner) if inner.len() >= 2 => {
                let id = match &inner[0] {
                    RencodeValue::Int(i) => u32::try_from(*i).unwrap_or(0),
                    _ => 0,
                };
                let method = match &inner[1] {
                    RencodeValue::Str(s) => s.clone(),
                    _ => String::new(),
                };
                let is_login = method == "daemon.login";
                (id, is_login, method)
            }
            _ => (0, false, String::new()),
        },
        _ => (0, false, String::new()),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn when_record_fails_then_cassette_not_modified() {
    let temp = TempDir::new().expect("temp dir");
    let cassette_path = temp.path().join("cassette.json");

    fs::write(&cassette_path, "existing content").expect("write existing");

    let mut cmd = Command::cargo_bin("deluge-cli").expect("binary exists");
    cmd.arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg("1")
        .arg("--password")
        .arg("dummy")
        .arg("--record")
        .arg(cassette_path.to_str().unwrap())
        .arg("daemon")
        .arg("info")
        .timeout(Duration::from_secs(10))
        .assert()
        .failure();

    let content = fs::read_to_string(&cassette_path).expect("read cassette");
    assert_eq!(
        content, "existing content",
        "cassette should not be modified on failure"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_record_with_existing_cassette_then_appends_interactions() {
    let temp = TempDir::new().expect("temp dir");
    let cassette_path = temp.path().join("cassette.json");

    let existing = serde_json::json!({
        "version": 1,
        "recorded_at": "2025-01-01T00:00:00Z",
        "interactions": [{
            "request": {
                "method": "core.get_free_space",
                "args": { "type": "list", "value": [] },
                "kwargs": { "type": "list", "value": [] }
            },
            "response": {
                "type": "ok",
                "value": { "type": "int", "value": 1073741824 }
            }
        }]
    });
    fs::write(
        &cassette_path,
        serde_json::to_string_pretty(&existing).expect("serialize"),
    )
    .expect("write existing");

    let mock = MockDaemon::new().await;

    let mut cmd = Command::cargo_bin("deluge-cli").expect("binary exists");
    cmd.arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(mock.addr.port().to_string())
        .arg("--password")
        .arg("dummy")
        .arg("--record")
        .arg(cassette_path.to_str().unwrap())
        .arg("daemon")
        .arg("info")
        .timeout(Duration::from_secs(10))
        .assert()
        .success();

    let content = fs::read_to_string(&cassette_path).expect("read cassette");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    let interactions = parsed["interactions"]
        .as_array()
        .expect("interactions array");
    assert_eq!(
        interactions.len(),
        2,
        "cassette should contain both the existing and new interaction"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_record_flag_then_cassette_written() {
    let temp = TempDir::new().expect("temp dir");
    let cassette_path = temp.path().join("cassette.json");

    let mock = MockDaemon::new().await;

    let mut cmd = Command::cargo_bin("deluge-cli").expect("binary exists");
    cmd.arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(mock.addr.port().to_string())
        .arg("--password")
        .arg("dummy")
        .arg("--record")
        .arg(cassette_path.to_str().unwrap())
        .arg("daemon")
        .arg("info")
        .timeout(Duration::from_secs(10))
        .assert()
        .success();

    assert!(cassette_path.exists(), "cassette should be created");

    let content = fs::read_to_string(&cassette_path).expect("read cassette");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert_eq!(parsed["version"], 1);
    let interactions = parsed["interactions"]
        .as_array()
        .expect("interactions array");
    assert!(
        !interactions.is_empty(),
        "should have at least one interaction"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_typed_free_space_then_pretty_output() {
    let mock = MockDaemon::new().await;

    let mut cmd = Command::cargo_bin("deluge-cli").expect("binary exists");
    let output = cmd
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(mock.addr.port().to_string())
        .arg("--password")
        .arg("dummy")
        .arg("core")
        .arg("free-space")
        .timeout(Duration::from_secs(10))
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf8");
    assert!(
        stdout.contains("1073741824"),
        "should contain free space bytes: {stdout}"
    );
}
