use crate::client::RpcCaller;
use crate::client::core::{
    CoreAccountClient, CoreConfigClient, CoreMiscClient, CorePluginClient, CoreSessionClient,
    CoreTorrentClient,
};
use crate::client::daemon::DaemonClient;
use crate::client::plugins::{
    AutoaddClient, BlocklistClient, ExecuteClient, ExtractorClient, LabelClient,
    NotificationsClient, SchedulerClient, StatsClient, ToggleClient, WebuiClient,
};
use crate::connection::reader_loop;
use crate::protocol::DelugeRpcMessage;
use crate::protocol::DelugeRpcRequest;
use crate::rencode::RencodeValue;
use crate::shared::Shared;
use crate::transport::{DelugeTransport, DelugeWriter};
use anyhow::{Context, anyhow};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{Mutex, broadcast};
use tokio::task::JoinHandle;
use tokio::time::timeout;

const BROADCAST_CAPACITY: usize = 256;
const RPC_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct DelugeClientInner {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub next_id: AtomicU32,
    pub state: Mutex<ConnectionState>,
}

impl DelugeClientInner {
    fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
            next_id: AtomicU32::new(1),
            state: Mutex::new(ConnectionState::Disconnected),
        }
    }
}

pub(crate) enum ConnectionState {
    Connected {
        shared: Arc<Shared>,
        writer: Arc<Mutex<DelugeWriter>>,
        reader_handle: JoinHandle<()>,
    },
    Disconnected,
}

impl ConnectionState {
    fn is_connected(&self) -> bool {
        match self {
            ConnectionState::Connected { reader_handle, .. } => !reader_handle.is_finished(),
            ConnectionState::Disconnected => false,
        }
    }

    pub(crate) async fn ensure_connected(
        &mut self,
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> anyhow::Result<()> {
        if self.is_connected() {
            return Ok(());
        }

        let transport = DelugeTransport::connect(host, port)
            .await
            .context("failed to connect to Deluge daemon")?;
        let (reader, writer) = transport.split();

        let shared = Shared::new(BROADCAST_CAPACITY);
        let writer = Arc::new(Mutex::new(writer));

        let reader_shared = shared.clone();
        let reader_handle = tokio::spawn(async move {
            reader_loop(reader, reader_shared).await;
        });

        let login_result = {
            let login_id = shared.next_id.fetch_add(1, Ordering::Relaxed);

            let mut kwargs = BTreeMap::new();
            kwargs.insert(
                RencodeValue::Str("client_version".into()),
                RencodeValue::Str(format!("deluge-rpc/{}", env!("CARGO_PKG_VERSION"))),
            );

            let login_request = DelugeRpcRequest::new("daemon.login")
                .with_args(vec![
                    RencodeValue::Str(username.to_owned()),
                    RencodeValue::Str(password.to_owned()),
                ])
                .with_kwargs(kwargs);
            let login_encoded = login_request.encode(login_id);

            let mut rx = shared.message_tx.subscribe();

            {
                let mut w = writer.lock().await;
                w.send(&login_encoded)
                    .await
                    .context("failed to send daemon.login")?;
            }

            timeout(RPC_TIMEOUT, async {
                loop {
                    match rx.recv().await {
                        Ok(DelugeRpcMessage::Response { id, .. }) if id == login_id => {
                            return Ok::<_, anyhow::Error>(());
                        }
                        Ok(DelugeRpcMessage::Error {
                            id,
                            exc_type,
                            exc_msg,
                            ..
                        }) if id == login_id => {
                            return Err(anyhow!("daemon.login failed: {exc_type}: {exc_msg}"));
                        }
                        Ok(_) => continue,
                        Err(RecvError::Lagged(n)) => {
                            tracing::warn!(n, "login subscriber lagged");
                            continue;
                        }
                        Err(RecvError::Closed) => {
                            return Err(anyhow!("connection closed during daemon.login"));
                        }
                    }
                }
            })
            .await
        };

        match login_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                reader_handle.abort();
                return Err(e);
            }
            Err(_) => {
                reader_handle.abort();
                return Err(anyhow!("timed out waiting for daemon.login response"));
            }
        }

        *self = ConnectionState::Connected {
            shared,
            writer,
            reader_handle,
        };

        Ok(())
    }

    pub(crate) fn writer_and_rx(
        &self,
    ) -> Option<(
        Arc<Mutex<DelugeWriter>>,
        broadcast::Receiver<DelugeRpcMessage>,
    )> {
        match self {
            ConnectionState::Connected { writer, shared, .. } => {
                let rx = shared.message_tx.subscribe();
                Some((Arc::clone(writer), rx))
            }
            ConnectionState::Disconnected => None,
        }
    }
}

pub struct DelugeClient {
    #[expect(dead_code, reason = "held for reconnect via RpcCaller")]
    inner: Arc<DelugeClientInner>,
    daemon_client: DaemonClient,
    core_client: CoreClient,
    plugins_client: PluginsClient,
}

impl DelugeClient {
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> anyhow::Result<Self> {
        let inner = Arc::new(DelugeClientInner::new(
            host.to_owned(),
            port,
            username.to_owned(),
            password.to_owned(),
        ));

        {
            let mut state = inner.state.lock().await;
            state
                .ensure_connected(host, port, username, password)
                .await
                .context("initial connect failed")?;
        }

        let caller = RpcCaller::new_reconnect(Arc::clone(&inner));
        let daemon_client = DaemonClient::new(caller.clone());
        let core_client = CoreClient::new(caller.clone());
        let plugins_client = PluginsClient::new(caller);

        Ok(Self {
            inner,
            daemon_client,
            core_client,
            plugins_client,
        })
    }

    pub fn daemon(&self) -> &DaemonClient {
        &self.daemon_client
    }

    pub fn core(&self) -> &CoreClient {
        &self.core_client
    }

    pub fn plugins(&self) -> &PluginsClient {
        &self.plugins_client
    }

    pub fn rpc_caller(&self) -> RpcCaller {
        self.daemon_client.rpc_caller()
    }
}

pub struct CoreClient {
    pub torrents: CoreTorrentClient,
    pub session: CoreSessionClient,
    pub config: CoreConfigClient,
    pub plugins: CorePluginClient,
    pub accounts: CoreAccountClient,
    pub misc: CoreMiscClient,
}

impl CoreClient {
    fn new(caller: RpcCaller) -> Self {
        Self {
            torrents: CoreTorrentClient::new(caller.clone()),
            session: CoreSessionClient::new(caller.clone()),
            config: CoreConfigClient::new(caller.clone()),
            plugins: CorePluginClient::new(caller.clone()),
            accounts: CoreAccountClient::new(caller.clone()),
            misc: CoreMiscClient::new(caller),
        }
    }
}

pub struct PluginsClient {
    pub autoadd: AutoaddClient,
    pub blocklist: BlocklistClient,
    pub execute: ExecuteClient,
    pub extractor: ExtractorClient,
    pub label: LabelClient,
    pub notifications: NotificationsClient,
    pub scheduler: SchedulerClient,
    pub stats: StatsClient,
    pub toggle: ToggleClient,
    pub webui: WebuiClient,
}

impl PluginsClient {
    fn new(caller: RpcCaller) -> Self {
        Self {
            autoadd: AutoaddClient::new(caller.clone()),
            blocklist: BlocklistClient::new(caller.clone()),
            execute: ExecuteClient::new(caller.clone()),
            extractor: ExtractorClient::new(caller.clone()),
            label: LabelClient::new(caller.clone()),
            notifications: NotificationsClient::new(caller.clone()),
            scheduler: SchedulerClient::new(caller.clone()),
            stats: StatsClient::new(caller.clone()),
            toggle: ToggleClient::new(caller.clone()),
            webui: WebuiClient::new(caller),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::daemon::DaemonRpc;
    use crate::rencode::RencodeValue;
    use flate2::Compression;
    use flate2::read::ZlibDecoder;
    use flate2::write::ZlibEncoder;
    use rustls::crypto::ring;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    use rustls::server::ServerConfig;
    use std::io::{Read, Write};
    use std::net::SocketAddr;
    use std::sync::Once;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::sleep;
    use tokio_rustls::server::TlsStream;

    const HEADER_LEN: usize = 5;
    const PROTOCOL_VERSION: u8 = 1;

    fn ensure_crypto_provider() {
        static INSTALL: Once = Once::new();
        INSTALL.call_once(|| {
            let _ = ring::default_provider().install_default();
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

    fn login_response_frame(request_id: u32) -> Vec<u8> {
        let response = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Int(1),
            RencodeValue::Int(i64::from(request_id)),
            RencodeValue::List(vec![RencodeValue::Int(10)]),
        ])]);
        let payload = response.encode();
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&payload).expect("compress");
        let compressed = enc.finish().expect("finish");
        let len = compressed.len() as u32;
        let mut frame = Vec::with_capacity(HEADER_LEN + compressed.len());
        frame.push(PROTOCOL_VERSION);
        frame.extend_from_slice(&len.to_be_bytes());
        frame.extend_from_slice(&compressed);
        frame
    }

    #[expect(dead_code, reason = "test helper available for future test scenarios")]
    fn echo_frame(data: &[u8]) -> Vec<u8> {
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(data).expect("compress");
        let compressed = enc.finish().expect("finish");
        let len = compressed.len() as u32;
        let mut frame = Vec::with_capacity(HEADER_LEN + compressed.len());
        frame.push(PROTOCOL_VERSION);
        frame.extend_from_slice(&len.to_be_bytes());
        frame.extend_from_slice(&compressed);
        frame
    }

    fn rpc_response_frame(request_id: u32, return_value: RencodeValue) -> Vec<u8> {
        let response = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Int(1),
            RencodeValue::Int(i64::from(request_id)),
            RencodeValue::List(vec![return_value]),
        ])]);
        let payload = response.encode();
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&payload).expect("compress");
        let compressed = enc.finish().expect("finish");
        let len = compressed.len() as u32;
        let mut frame = Vec::with_capacity(HEADER_LEN + compressed.len());
        frame.push(PROTOCOL_VERSION);
        frame.extend_from_slice(&len.to_be_bytes());
        frame.extend_from_slice(&compressed);
        frame
    }

    struct MockServer {
        addr: SocketAddr,
    }

    impl MockServer {
        async fn new(drop_after_requests: u32) -> Self {
            ensure_crypto_provider();
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
                        handle_connection(tls, drop_after_requests).await;
                    });
                }
            });

            MockServer { addr }
        }
    }

    async fn handle_connection(mut tls: TlsStream<TcpStream>, drop_after_requests: u32) {
        let mut header = [0u8; HEADER_LEN];
        let mut request_count: u32 = 0;
        loop {
            if tls.read_exact(&mut header).await.is_err() {
                break;
            }
            let body_len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
            let mut body = vec![0u8; body_len as usize];
            if tls.read_exact(&mut body).await.is_err() {
                break;
            }

            let decoded = {
                let mut dec = ZlibDecoder::new(&body[..]);
                let mut out = Vec::new();
                let _ = dec.read_to_end(&mut out);
                out
            };

            request_count += 1;

            let (request_id, is_login) = {
                let value = RencodeValue::decode(&decoded).ok();
                value
                    .and_then(|v| match v {
                        RencodeValue::List(mut items) if items.len() == 1 => {
                            match items.remove(0) {
                                RencodeValue::List(inner) if inner.len() >= 2 => {
                                    let id = match &inner[0] {
                                        RencodeValue::Int(i) => u32::try_from(*i).ok(),
                                        _ => None,
                                    };
                                    let login = match &inner[1] {
                                        RencodeValue::Str(s) => s == "daemon.login",
                                        _ => false,
                                    };
                                    id.map(|i| (i, login))
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .unwrap_or((0, false))
            };

            if is_login {
                let response_frame = login_response_frame(request_id);
                if tls.write_all(&response_frame).await.is_err() {
                    break;
                }
                let _ = tls.flush().await;
            } else {
                let response_frame = rpc_response_frame(request_id, RencodeValue::Str("ok".into()));
                if tls.write_all(&response_frame).await.is_err() {
                    break;
                }
                let _ = tls.flush().await;
            }

            if request_count >= drop_after_requests {
                let _ = tls.shutdown().await;
                break;
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_connect_then_client_has_all_sub_clients() {
        let server = MockServer::new(u32::MAX).await;

        let client = DelugeClient::connect("127.0.0.1", server.addr.port(), "testuser", "testpass")
            .await
            .expect("connect");

        let _daemon = client.daemon();
        let core = client.core();
        let _torrents = &core.torrents;
        let _session = &core.session;
        let _config = &core.config;
        let _core_plugins = &core.plugins;
        let _accounts = &core.accounts;
        let _misc = &core.misc;

        let plugins = client.plugins();
        let _autoadd = &plugins.autoadd;
        let _blocklist = &plugins.blocklist;
        let _execute = &plugins.execute;
        let _extractor = &plugins.extractor;
        let _label = &plugins.label;
        let _notifications = &plugins.notifications;
        let _scheduler = &plugins.scheduler;
        let _stats = &plugins.stats;
        let _toggle = &plugins.toggle;
        let _webui = &plugins.webui;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_reader_dies_then_next_call_reconnects() {
        let server = MockServer::new(2).await;

        let client = DelugeClient::connect("127.0.0.1", server.addr.port(), "testuser", "testpass")
            .await
            .expect("connect");

        let result = client.daemon().info().await;
        assert!(result.is_ok(), "first call should succeed: {result:?}");

        sleep(Duration::from_millis(100)).await;

        let result2 = client.daemon().info().await;
        assert!(
            result2.is_ok(),
            "second call after reconnect should succeed: {result2:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_reconnect_then_re_login_succeeds() {
        let server = MockServer::new(2).await;

        let client = DelugeClient::connect("127.0.0.1", server.addr.port(), "testuser", "testpass")
            .await
            .expect("connect");

        let _ = client.daemon().info().await;

        sleep(Duration::from_millis(100)).await;

        let result = client.daemon().get_version().await;
        assert!(
            result.is_ok(),
            "call after reconnect should succeed: {result:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_in_flight_request_dropped_then_error_returned() {
        let server = MockServer::new(1).await;

        let client = DelugeClient::connect("127.0.0.1", server.addr.port(), "testuser", "testpass")
            .await
            .expect("connect");

        sleep(Duration::from_millis(100)).await;

        let result = client.daemon().info().await;
        assert!(
            result.is_err(),
            "call should fail (connection dropped): {result:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_repeated_reconnects_then_no_reader_leak() {
        let server = MockServer::new(2).await;

        let client = DelugeClient::connect("127.0.0.1", server.addr.port(), "testuser", "testpass")
            .await
            .expect("connect");

        for i in 0..5 {
            let result = client.daemon().info().await;
            assert!(result.is_ok(), "call {i} should succeed: {result:?}");
            sleep(Duration::from_millis(50)).await;
        }
    }
}
