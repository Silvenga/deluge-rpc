use crate::client::dispatcher::DelugeClientDispatcher;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::{extract_single, extract_single_int};
use crate::RencodeValue;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use std::collections::BTreeMap;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait DaemonRpc: Send + Sync {
    async fn info(&self) -> anyhow::Result<String>;
    async fn login(
        &self,
        username: &str,
        password: &str,
        client_version: &str,
    ) -> anyhow::Result<i64>;
    async fn set_event_interest(&self, event_names: &[String]) -> anyhow::Result<bool>;
    async fn shutdown(&self) -> anyhow::Result<()>;
    async fn get_method_list(&self) -> anyhow::Result<Vec<String>>;
    async fn get_version(&self) -> anyhow::Result<String>;
    async fn authorized_call(&self, rpc: &str) -> anyhow::Result<bool>;
}

pub struct DaemonClient {
    dispatcher: DelugeClientDispatcher,
}

impl DaemonClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl DaemonRpc for DaemonClient {
    async fn info(&self) -> anyhow::Result<String> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("daemon.info"))
            .await
            .context("daemon.info RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(anyhow!("daemon.info returned non-str value: {other:?}")),
        }
    }

    async fn login(
        &self,
        username: &str,
        password: &str,
        client_version: &str,
    ) -> anyhow::Result<i64> {
        let mut kwargs = BTreeMap::new();
        kwargs.insert(
            RencodeValue::Str("client_version".into()),
            RencodeValue::Str(client_version.to_owned()),
        );
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("daemon.login")
                    .with_args(vec![
                        RencodeValue::Str(username.to_owned()),
                        RencodeValue::Str(password.to_owned()),
                    ])
                    .with_kwargs(kwargs),
            )
            .await
            .context("daemon.login RPC failed")?;
        extract_single_int(&result, "daemon.login")
    }

    async fn set_event_interest(&self, event_names: &[String]) -> anyhow::Result<bool> {
        let names: Vec<RencodeValue> = event_names
            .iter()
            .map(|n| RencodeValue::Str(n.clone()))
            .collect();
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("daemon.set_event_interest")
                    .with_args(vec![RencodeValue::List(names)]),
            )
            .await
            .context("daemon.set_event_interest RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "daemon.set_event_interest returned non-bool value: {other:?}"
            )),
        }
    }

    /// Shuts down the daemon. The daemon's reactor stops before the response is flushed,
    /// so this call may time out (30s) waiting for a response that never arrives.
    /// This is expected behavior per the Deluge spec — do not reduce the timeout.
    async fn shutdown(&self) -> anyhow::Result<()> {
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("daemon.shutdown"))
            .await
            .context("daemon.shutdown RPC failed")?;
        Ok(())
    }

    async fn get_method_list(&self) -> anyhow::Result<Vec<String>> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("daemon.get_method_list"))
            .await
            .context("daemon.get_method_list RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        RencodeValue::Str(s) => out.push(s),
                        other => {
                            return Err(anyhow!(
                                "daemon.get_method_list returned non-str element: {other:?}"
                            ));
                        }
                    }
                }
                Ok(out)
            }
            other => Err(anyhow!(
                "daemon.get_method_list returned non-list value: {other:?}"
            )),
        }
    }

    async fn get_version(&self) -> anyhow::Result<String> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("daemon.get_version"))
            .await
            .context("daemon.get_version RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(anyhow!(
                "daemon.get_version returned non-str value: {other:?}"
            )),
        }
    }

    async fn authorized_call(&self, rpc: &str) -> anyhow::Result<bool> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("daemon.authorized_call")
                    .with_args(vec![RencodeValue::Str(rpc.to_owned())]),
            )
            .await
            .context("daemon.authorized_call RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "daemon.authorized_call returned non-bool value: {other:?}"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;

    #[test]
    fn when_daemon_info_then_string() {
        let response = RencodeValue::Str("2.1.1".into());
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Str(s) => assert_eq!(s, "2.1.1"),
            other => panic!("expected str, got {other:?}"),
        }
    }

    #[test]
    fn when_daemon_login_then_int() {
        let response = RencodeValue::Int(10);
        let level = extract_single_int(&response, "daemon.login").expect("extract");
        assert_eq!(level, 10);
    }

    #[test]
    fn when_daemon_set_event_interest_then_bool() {
        let response = RencodeValue::Bool(true);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[test]
    fn when_daemon_get_method_list_then_vec_string() {
        let response = RencodeValue::List(vec![
            RencodeValue::Str("core.add_torrent_file".into()),
            RencodeValue::Str("core.get_free_space".into()),
        ]);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::List(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], RencodeValue::Str("core.add_torrent_file".into()));
                assert_eq!(items[1], RencodeValue::Str("core.get_free_space".into()));
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn when_daemon_get_version_then_string() {
        let response = RencodeValue::Str("2.1.1".into());
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Str(s) => assert_eq!(s, "2.1.1"),
            other => panic!("expected str, got {other:?}"),
        }
    }

    #[test]
    fn when_daemon_authorized_call_then_bool() {
        let response = RencodeValue::Bool(true);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[cfg(feature = "mock")]
    #[test]
    fn when_mock_daemon_rpc_then_expectations_met() {
        use tokio::runtime::Runtime;

        let mut mock = MockDaemonRpc::new();
        mock.expect_info().times(1).returning(|| Ok("2.1.1".into()));
        mock.expect_login().times(1).returning(|_, _, _| Ok(10));
        mock.expect_set_event_interest()
            .times(1)
            .returning(|_| Ok(true));
        mock.expect_get_version()
            .times(1)
            .returning(|| Ok("2.1.1".into()));
        mock.expect_authorized_call()
            .times(1)
            .returning(|_| Ok(true));

        let rt = Runtime::new().expect("runtime");
        rt.block_on(async {
            assert_eq!(mock.info().await.expect("info"), "2.1.1");
            assert_eq!(mock.login("user", "pass", "1.0").await.expect("login"), 10);
            assert!(
                mock.set_event_interest(&["TorrentAddedEvent".into()])
                    .await
                    .expect("set_event_interest")
            );
            assert_eq!(mock.get_version().await.expect("get_version"), "2.1.1");
            assert!(
                mock.authorized_call("core.get_free_space")
                    .await
                    .expect("authorized_call")
            );
        });
    }
}
