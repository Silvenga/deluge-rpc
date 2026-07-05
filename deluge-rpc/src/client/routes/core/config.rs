use crate::client::caller::RpcCaller;
use crate::models::{DaemonConfig, ProxyConfig};
use crate::protocol::extract_single;
use crate::protocol::DelugeRpcRequest;
use crate::rencode::RencodeValue;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::BTreeMap;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait CoreConfigRpc: Send + Sync {
    async fn get_config(&self) -> anyhow::Result<DaemonConfig>;
    async fn get_config_value(&self, key: &str) -> anyhow::Result<RencodeValue>;
    async fn get_config_values(
        &self,
        keys: &[String],
    ) -> anyhow::Result<BTreeMap<String, RencodeValue>>;
    async fn set_config(&self, config: &BTreeMap<String, RencodeValue>) -> anyhow::Result<()>;
    async fn get_proxy(&self) -> anyhow::Result<ProxyConfig>;
}

pub struct CoreConfigClient {
    caller: RpcCaller,
}

impl CoreConfigClient {
    pub(crate) fn new(caller: RpcCaller) -> Self {
        Self { caller }
    }
}

impl Clone for CoreConfigClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl CoreConfigRpc for CoreConfigClient {
    async fn get_config(&self) -> anyhow::Result<DaemonConfig> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_config"))
            .await
            .context("core.get_config RPC failed")?;
        let value = extract_single(&result)?;
        DaemonConfig::deserialize(&value).context("deserializing daemon config")
    }

    async fn get_config_value(&self, key: &str) -> anyhow::Result<RencodeValue> {
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.get_config_value")
                    .with_args(vec![RencodeValue::Str(key.to_owned())]),
            )
            .await
            .context("core.get_config_value RPC failed")?;
        extract_single(&result)
    }

    async fn get_config_values(
        &self,
        keys: &[String],
    ) -> anyhow::Result<BTreeMap<String, RencodeValue>> {
        let key_values: Vec<RencodeValue> =
            keys.iter().map(|k| RencodeValue::Str(k.clone())).collect();
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.get_config_values")
                    .with_args(vec![RencodeValue::List(key_values)]),
            )
            .await
            .context("core.get_config_values RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Dict(map) => {
                let mut out = BTreeMap::new();
                for (k, v) in map {
                    match k {
                        RencodeValue::Str(s) => {
                            out.insert(s, v);
                        }
                        other => {
                            return Err(anyhow!(
                                "core.get_config_values returned non-str key: {other:?}"
                            ));
                        }
                    }
                }
                Ok(out)
            }
            other => Err(anyhow!(
                "core.get_config_values returned non-dict value: {other:?}"
            )),
        }
    }

    async fn set_config(&self, config: &BTreeMap<String, RencodeValue>) -> anyhow::Result<()> {
        let config_dict: BTreeMap<RencodeValue, RencodeValue> = config
            .iter()
            .map(|(k, v)| (RencodeValue::Str(k.clone()), v.clone()))
            .collect();
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("core.set_config")
                    .with_args(vec![RencodeValue::Dict(config_dict)]),
            )
            .await
            .context("core.set_config RPC failed")?;
        Ok(())
    }

    async fn get_proxy(&self) -> anyhow::Result<ProxyConfig> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_proxy"))
            .await
            .context("core.get_proxy RPC failed")?;
        let value = extract_single(&result)?;
        ProxyConfig::deserialize(&value).context("deserializing proxy config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;
    use std::collections::BTreeMap;

    #[test]
    fn when_core_get_config_value_then_rencode_value() {
        let response = RencodeValue::Str("/downloads".into());
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Str(s) => assert_eq!(s, "/downloads"),
            other => panic!("expected str, got {other:?}"),
        }
    }

    #[test]
    fn when_core_get_config_values_then_dict() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("download_location".into()),
            RencodeValue::Str("/downloads".into()),
        );
        map.insert(
            RencodeValue::Str("daemon_port".into()),
            RencodeValue::Int(58846),
        );
        let response = RencodeValue::Dict(map);

        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Dict(m) => {
                assert_eq!(m.len(), 2);
                assert_eq!(
                    m.get(&RencodeValue::Str("download_location".into())),
                    Some(&RencodeValue::Str("/downloads".into()))
                );
            }
            other => panic!("expected dict, got {other:?}"),
        }
    }
}
