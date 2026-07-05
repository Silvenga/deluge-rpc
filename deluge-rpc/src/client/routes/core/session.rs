use crate::client::caller::RpcCaller;
use crate::models::SessionStatus;
use crate::protocol::{extract_single, extract_single_int, DelugeRpcRequest};
use crate::rencode::RencodeValue;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use serde::Deserialize;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait CoreSessionRpc: Send + Sync {
    async fn pause_session(&self) -> anyhow::Result<()>;
    async fn resume_session(&self) -> anyhow::Result<()>;
    async fn is_session_paused(&self) -> anyhow::Result<bool>;
    async fn get_listen_port(&self) -> anyhow::Result<i64>;
    /// Returns the active SSL listen port. This method may not exist on all daemon versions.
    async fn get_ssl_listen_port(&self) -> anyhow::Result<i64>;
    async fn get_external_ip(&self) -> anyhow::Result<String>;
    async fn get_libtorrent_version(&self) -> anyhow::Result<String>;
    async fn test_listen_port(&self) -> anyhow::Result<Option<bool>>;
    /// Returns libtorrent session statistics for the requested keys.
    ///
    /// `keys` is a required positional argument. Pass an empty slice `&[]`
    /// to request all available keys. The daemon returns a flat dict of
    /// `{key: value}` where values are `int` or `float`.
    async fn get_session_status(&self, keys: &[String]) -> anyhow::Result<SessionStatus>;
    async fn get_free_space(&self, path: Option<String>) -> anyhow::Result<i64>;
}

pub struct CoreSessionClient {
    caller: RpcCaller,
}

impl CoreSessionClient {
    pub(crate) fn new(caller: RpcCaller) -> Self {
        Self { caller }
    }
}

impl Clone for CoreSessionClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl CoreSessionRpc for CoreSessionClient {
    async fn pause_session(&self) -> anyhow::Result<()> {
        self.caller
            .rpc_call(DelugeRpcRequest::new("core.pause_session"))
            .await
            .context("core.pause_session RPC failed")?;
        Ok(())
    }

    async fn resume_session(&self) -> anyhow::Result<()> {
        self.caller
            .rpc_call(DelugeRpcRequest::new("core.resume_session"))
            .await
            .context("core.resume_session RPC failed")?;
        Ok(())
    }

    async fn is_session_paused(&self) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.is_session_paused"))
            .await
            .context("core.is_session_paused RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "core.is_session_paused returned non-bool value: {other:?}"
            )),
        }
    }

    async fn get_listen_port(&self) -> anyhow::Result<i64> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_listen_port"))
            .await
            .context("core.get_listen_port RPC failed")?;
        extract_single_int(&result, "core.get_listen_port")
    }

    async fn get_ssl_listen_port(&self) -> anyhow::Result<i64> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_ssl_listen_port"))
            .await
            .context("core.get_ssl_listen_port RPC failed")?;
        extract_single_int(&result, "core.get_ssl_listen_port")
    }

    async fn get_external_ip(&self) -> anyhow::Result<String> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_external_ip"))
            .await
            .context("core.get_external_ip RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(anyhow!(
                "core.get_external_ip returned non-str value: {other:?}"
            )),
        }
    }

    async fn get_libtorrent_version(&self) -> anyhow::Result<String> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_libtorrent_version"))
            .await
            .context("core.get_libtorrent_version RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Str(s) => Ok(s),
            other => Err(anyhow!(
                "core.get_libtorrent_version returned non-str value: {other:?}"
            )),
        }
    }

    /// Tests whether the active listen port is open by making an HTTP request to a Deluge
    /// test service. This call may be slow (network-dependent) and can return None on error.
    async fn test_listen_port(&self) -> anyhow::Result<Option<bool>> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.test_listen_port"))
            .await
            .context("core.test_listen_port RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(Some(b)),
            RencodeValue::None => Ok(None),
            other => Err(anyhow!(
                "core.test_listen_port returned unexpected value: {other:?}"
            )),
        }
    }

    async fn get_session_status(&self, keys: &[String]) -> anyhow::Result<SessionStatus> {
        let key_values: Vec<RencodeValue> =
            keys.iter().map(|k| RencodeValue::Str(k.clone())).collect();
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.get_session_status")
                    .with_args(vec![RencodeValue::List(key_values)]),
            )
            .await
            .context("core.get_session_status RPC failed")?;
        let value = extract_single(&result)?;
        SessionStatus::deserialize(&value).context("deserializing session status")
    }

    async fn get_free_space(&self, path: Option<String>) -> anyhow::Result<i64> {
        let args = match path {
            Some(p) => vec![RencodeValue::Str(p)],
            None => vec![RencodeValue::None],
        };
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_free_space").with_args(args))
            .await
            .context("core.get_free_space RPC failed")?;
        extract_single_int(&result, "core.get_free_space")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;
    use std::collections::BTreeMap;

    #[test]
    fn when_core_get_free_space_then_i64() {
        let response = RencodeValue::Int(1_073_741_824);
        let bytes = extract_single_int(&response, "core.get_free_space").expect("extract");
        assert_eq!(bytes, 1_073_741_824);
    }

    #[test]
    fn when_core_is_session_paused_then_bool() {
        let response = RencodeValue::Bool(false);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(!b),
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[test]
    fn when_core_get_listen_port_then_int() {
        let response = RencodeValue::Int(6881);
        let port = extract_single_int(&response, "core.get_listen_port").expect("extract");
        assert_eq!(port, 6881);
    }

    #[test]
    fn when_core_get_external_ip_then_string() {
        let response = RencodeValue::Str("1.2.3.4".into());
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Str(s) => assert_eq!(s, "1.2.3.4"),
            other => panic!("expected str, got {other:?}"),
        }
    }

    #[test]
    fn when_core_test_listen_port_some_then_bool() {
        let response = RencodeValue::Bool(true);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[test]
    fn when_core_test_listen_port_none_then_none() {
        let response = RencodeValue::None;
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::None => {}
            other => panic!("expected None, got {other:?}"),
        }
    }

    #[test]
    fn when_core_get_session_status_then_session_status() {
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
        let response = RencodeValue::Dict(map);

        let value = extract_single(&response).expect("extract");
        let status: SessionStatus = SessionStatus::deserialize(&value).expect("deserialize");

        assert!((status.download_rate - 1024.0).abs() < f64::EPSILON);
        assert!((status.upload_rate - 512.0).abs() < f64::EPSILON);
        assert!((status.write_hit_ratio - 0.95).abs() < f64::EPSILON);
        assert!((status.read_hit_ratio - 0.88).abs() < f64::EPSILON);
    }
}
