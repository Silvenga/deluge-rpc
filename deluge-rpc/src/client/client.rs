use crate::client::dispatcher::DelugeClientDispatcher;
use crate::client::info::DelugeConnectionInfo;
use crate::{
    AutoAddClient, BlocklistClient, CoreAccountClient, CoreConfigClient, CoreMiscClient,
    CorePluginClient, CoreSessionClient, CoreTorrentClient, DaemonClient, DelugeRpcRequest,
    ExecuteClient, ExtractorClient, LabelClient, NotificationsClient, RencodeValue,
    SchedulerClient, StatsClient, ToggleClient, WebUiClient,
};

pub struct DelugeClient {
    dispatcher: DelugeClientDispatcher,
    daemon_client: DaemonClient,
    core_client: CoreClient,
    plugins_client: PluginsClient,
}

impl DelugeClient {
    pub fn new(info: DelugeConnectionInfo) -> Self {
        let dispatcher = DelugeClientDispatcher::new(info.into());
        Self {
            daemon_client: DaemonClient::new(dispatcher.clone()),
            core_client: CoreClient::new(dispatcher.clone()),
            plugins_client: PluginsClient::new(dispatcher.clone()),
            dispatcher,
        }
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

    /// A low-level method to call the RPC server directly.
    pub async fn call(&self, request: DelugeRpcRequest) -> anyhow::Result<RencodeValue> {
        self.dispatcher.dispatch(request).await
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
    fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self {
            torrents: CoreTorrentClient::new(dispatcher.clone()),
            session: CoreSessionClient::new(dispatcher.clone()),
            config: CoreConfigClient::new(dispatcher.clone()),
            plugins: CorePluginClient::new(dispatcher.clone()),
            accounts: CoreAccountClient::new(dispatcher.clone()),
            misc: CoreMiscClient::new(dispatcher),
        }
    }
}

pub struct PluginsClient {
    pub auto_add: AutoAddClient,
    pub blocklist: BlocklistClient,
    pub execute: ExecuteClient,
    pub extractor: ExtractorClient,
    pub label: LabelClient,
    pub notifications: NotificationsClient,
    pub scheduler: SchedulerClient,
    pub stats: StatsClient,
    pub toggle: ToggleClient,
    pub webui: WebUiClient,
}

impl PluginsClient {
    fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self {
            auto_add: AutoAddClient::new(dispatcher.clone()),
            blocklist: BlocklistClient::new(dispatcher.clone()),
            execute: ExecuteClient::new(dispatcher.clone()),
            extractor: ExtractorClient::new(dispatcher.clone()),
            label: LabelClient::new(dispatcher.clone()),
            notifications: NotificationsClient::new(dispatcher.clone()),
            scheduler: SchedulerClient::new(dispatcher.clone()),
            stats: StatsClient::new(dispatcher.clone()),
            toggle: ToggleClient::new(dispatcher.clone()),
            webui: WebUiClient::new(dispatcher),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::DaemonRpc;
    use crate::rencode::RencodeValue;
    use crate::DelugeClientBuilder;
    use flate2::read::ZlibDecoder;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use rustls::crypto::ring;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    use rustls::server::ServerConfig;
    use std::io::{Read, Write};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::sync::Once;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
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
        let response = RencodeValue::List(vec![
            RencodeValue::Int(1),
            RencodeValue::Int(i64::from(request_id)),
            RencodeValue::Int(10),
        ]);
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

    fn rpc_response_frame(request_id: u32, return_value: RencodeValue) -> Vec<u8> {
        let response = RencodeValue::List(vec![
            RencodeValue::Int(1),
            RencodeValue::Int(i64::from(request_id)),
            return_value,
        ]);
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

        let client = DelugeClientBuilder::new(
            "127.0.0.1".to_owned(),
            server.addr.port(),
            "testuser".to_owned(),
            "testpass".to_owned(),
        )
        .build();

        let _daemon = client.daemon();
        let core = client.core();
        let _torrents = &core.torrents;
        let _session = &core.session;
        let _config = &core.config;
        let _core_plugins = &core.plugins;
        let _accounts = &core.accounts;
        let _misc = &core.misc;

        let plugins = client.plugins();
        let _auto_add = &plugins.auto_add;
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

        let client = DelugeClientBuilder::new(
            "127.0.0.1".to_owned(),
            server.addr.port(),
            "testuser".to_owned(),
            "testpass".to_owned(),
        )
        .build();

        let result = client.daemon().info().await;
        assert!(result.is_ok(), "first call should succeed: {result:?}");

        let result2 = client.daemon().info().await;
        assert!(
            result2.is_ok(),
            "second call after reconnect should succeed: {result2:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_reconnect_then_re_login_succeeds() {
        let server = MockServer::new(2).await;

        let client = DelugeClientBuilder::new(
            "127.0.0.1".to_owned(),
            server.addr.port(),
            "testuser".to_owned(),
            "testpass".to_owned(),
        )
        .build();

        let _ = client.daemon().info().await;

        let result = client.daemon().get_version().await;
        assert!(
            result.is_ok(),
            "call after reconnect should succeed: {result:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_in_flight_request_dropped_then_error_returned() {
        let server = MockServer::new(1).await;

        let client = DelugeClientBuilder::new(
            "127.0.0.1".to_owned(),
            server.addr.port(),
            "testuser".to_owned(),
            "testpass".to_owned(),
        )
        .build();

        let result = client.daemon().info().await;
        assert!(
            result.is_err(),
            "call should fail (connection dropped): {result:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_repeated_reconnects_then_no_reader_leak() {
        let server = MockServer::new(2).await;

        let client = DelugeClientBuilder::new(
            "127.0.0.1".to_owned(),
            server.addr.port(),
            "testuser".to_owned(),
            "testpass".to_owned(),
        )
        .build();

        for i in 0..5 {
            let result = client.daemon().info().await;
            assert!(result.is_ok(), "call {i} should succeed: {result:?}");
        }
    }
}
