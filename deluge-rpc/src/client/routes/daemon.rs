use crate::DelugeRpcError;
use crate::RencodeValue;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::{extract_single, extract_single_int};
use async_trait::async_trait;
use std::collections::BTreeMap;

#[async_trait]
pub trait DaemonRpc: Send + Sync {
    async fn info(&self) -> Result<String, DelugeRpcError>;
    async fn login(
        &self,
        username: &str,
        password: &str,
        client_version: &str,
    ) -> Result<i64, DelugeRpcError>;
    async fn set_event_interest(&self, event_names: &[String]) -> Result<bool, DelugeRpcError>;
    async fn shutdown(&self) -> Result<(), DelugeRpcError>;
    async fn get_method_list(&self) -> Result<Vec<String>, DelugeRpcError>;
    async fn get_version(&self) -> Result<String, DelugeRpcError>;
    async fn authorized_call(&self, rpc: &str) -> Result<bool, DelugeRpcError>;
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
    async fn info(&self) -> Result<String, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("daemon.info"))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "daemon.info".into(),
                value: other,
            }),
        }
    }

    async fn login(
        &self,
        username: &str,
        password: &str,
        client_version: &str,
    ) -> Result<i64, DelugeRpcError> {
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
            .await?;
        Ok(extract_single_int(&result, "daemon.login")?)
    }

    async fn set_event_interest(&self, event_names: &[String]) -> Result<bool, DelugeRpcError> {
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
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "daemon.set_event_interest".into(),
                value: other,
            }),
        }
    }

    /// Shuts down the daemon. The daemon's reactor stops before the response is flushed,
    /// so this call may time out (30s) waiting for a response that never arrives.
    /// This is expected behavior per the Deluge spec — do not reduce the timeout.
    async fn shutdown(&self) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("daemon.shutdown"))
            .await?;
        Ok(())
    }

    async fn get_method_list(&self) -> Result<Vec<String>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("daemon.get_method_list"))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        RencodeValue::Str(s) => out.push(s),
                        other => {
                            return Err(DelugeRpcError::UnexpectedResponseType {
                                method: "daemon.get_method_list returned non-str element".into(),
                                value: other,
                            });
                        }
                    }
                }
                Ok(out)
            }
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "daemon.get_method_list".into(),
                value: other,
            }),
        }
    }

    async fn get_version(&self) -> Result<String, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("daemon.get_version"))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "daemon.get_version".into(),
                value: other,
            }),
        }
    }

    async fn authorized_call(&self, rpc: &str) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("daemon.authorized_call")
                    .with_args(vec![RencodeValue::Str(rpc.to_owned())]),
            )
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "daemon.authorized_call".into(),
                value: other,
            }),
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
}
