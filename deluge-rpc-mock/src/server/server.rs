use crate::server::connection::handle_connection;
use crate::server::helpers::self_signed_server_config;
use crate::server::matcher::Matcher;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;

pub struct ReplayServer {
    addr: SocketAddr,
    handle: JoinHandle<()>,
    matcher: Arc<Matcher>,
}

impl ReplayServer {
    pub async fn start(matcher: Matcher) -> Result<Self, ReplayServerStartError> {
        let matcher = Arc::from(matcher);
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let acceptor = TlsAcceptor::from(Arc::from(self_signed_server_config()));

        let handle = tokio::spawn({
            let matcher_clone = matcher.clone();
            async move {
                loop {
                    let (tcp, _) = match listener.accept().await {
                        Ok(conn) => conn,
                        Err(_) => break,
                    };
                    let tls = match acceptor.accept(tcp).await {
                        Ok(tls) => tls,
                        Err(_) => continue,
                    };
                    tokio::spawn({
                        let m = matcher_clone.clone();
                        async move {
                            handle_connection(tls, &m).await;
                        }
                    });
                }
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

#[derive(Debug, thiserror::Error)]
pub enum ReplayServerStartError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Cassette, Interaction, InteractionRequest, InteractionResponse as CassetteResponse,
    };
    use deluge_rpc::DaemonRpc;
    use deluge_rpc::DelugeClientBuilder;
    use deluge_rpc::RencodeValue;

    fn make_cassette(interactions: Vec<Interaction>) -> Cassette {
        Cassette {
            version: 1,
            recorded_at: "2026-07-04T12:00:00Z".into(),
            daemon_version: None,
            interactions,
        }
    }

    fn ok_interaction(method: &str, args: RencodeValue, value: RencodeValue) -> Interaction {
        Interaction {
            request: InteractionRequest {
                method: method.into(),
                args,
                kwargs: RencodeValue::List(vec![]),
            },
            response: CassetteResponse::Ok { value },
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_cassette_loaded_then_replay_matches_exact_args() {
        let cassette = make_cassette(vec![ok_interaction(
            "daemon.info",
            RencodeValue::List(vec![RencodeValue::None]),
            RencodeValue::Str("2.1.1".into()),
        )]);
        let matcher = Matcher::new(cassette.interactions);
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClientBuilder::new(
            server.host(),
            server.port(),
            "testuser".to_owned(),
            "testpass".to_owned(),
        )
        .build();

        let result = client.daemon().info().await;
        assert!(result.is_ok(), "daemon.info should succeed: {result:?}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_no_exact_match_then_fallback_to_unmatched_for_method() {
        let cassette = make_cassette(vec![
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
        let matcher = Matcher::new(cassette.interactions);
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClientBuilder::new(
            server.host(),
            server.port(),
            "testuser".to_owned(),
            "testpass".to_owned(),
        )
        .build();

        let result = client.daemon().get_version().await;
        assert!(
            result.is_ok(),
            "first call (fallback) should succeed: {result:?}"
        );
        assert_eq!(result.unwrap(), "version-one");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_entry_consumed_then_not_rematched() {
        let cassette = make_cassette(vec![ok_interaction(
            "daemon.info",
            RencodeValue::List(vec![RencodeValue::None]),
            RencodeValue::Str("only-one".into()),
        )]);
        let matcher = Matcher::new(cassette.interactions);
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClientBuilder::new(
            server.host(),
            server.port(),
            "testuser".to_owned(),
            "testpass".to_owned(),
        )
        .build();

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
        let cassette = make_cassette(vec![Interaction {
            request: InteractionRequest {
                method: "daemon.info".into(),
                args: RencodeValue::List(vec![RencodeValue::None]),
                kwargs: RencodeValue::List(vec![]),
            },
            response: CassetteResponse::Error {
                exc_type: "TestError".into(),
                exc_msg: "this is a test error".into(),
                traceback: String::new(),
            },
        }]);
        let matcher = Matcher::new(cassette.interactions);
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClientBuilder::new(
            server.host(),
            server.port(),
            "testuser".to_owned(),
            "testpass".to_owned(),
        )
        .build();

        let result = client.daemon().info().await;
        assert!(result.is_err(), "should receive error: {result:?}");
        let err_msg = format!("{result:?}");
        assert!(
            err_msg.contains("TestError"),
            "error should mention TestError, got: {err_msg}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_login_auto_served_then_connect_succeeds_with_any_credentials() {
        let cassette = make_cassette(vec![]);
        let matcher = Matcher::new(cassette.interactions);
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClientBuilder::new(
            server.host(),
            server.port(),
            "anyuser".to_owned(),
            "anypassword".to_owned(),
        )
        .build();

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
        let cassette = make_cassette(vec![]);
        let matcher = Matcher::new(cassette.interactions);
        let server = ReplayServer::start(matcher).await.expect("start server");

        let client = DelugeClientBuilder::new(
            server.host(),
            server.port(),
            "testuser".to_owned(),
            "testpass".to_owned(),
        )
        .build();

        let result = client.daemon().info().await;
        assert!(result.is_err(), "unknown method should error: {result:?}");
        let err_msg = format!("{result:?}");
        assert!(
            err_msg.contains("UnknownMethod"),
            "error should contain UnknownMethod, got: {err_msg}"
        );
    }
}
