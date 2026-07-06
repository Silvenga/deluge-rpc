use crate::DelugeRpcError;
use crate::RencodeValue;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{DaemonConfig, ProxyConfig};
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::BTreeMap;

#[async_trait]
pub trait CoreConfigRpc: Send + Sync {
    async fn get_config(&self) -> Result<DaemonConfig, DelugeRpcError>;
    async fn get_config_value(&self, key: &str) -> Result<RencodeValue, DelugeRpcError>;
    async fn get_config_values(
        &self,
        keys: &[String],
    ) -> Result<BTreeMap<String, RencodeValue>, DelugeRpcError>;
    async fn set_config(
        &self,
        config: &BTreeMap<String, RencodeValue>,
    ) -> Result<(), DelugeRpcError>;
    async fn get_proxy(&self) -> Result<ProxyConfig, DelugeRpcError>;
}

pub struct CoreConfigClient {
    dispatcher: DelugeClientDispatcher,
}

impl CoreConfigClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for CoreConfigClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl CoreConfigRpc for CoreConfigClient {
    async fn get_config(&self) -> Result<DaemonConfig, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.get_config"))
            .await?;
        let value = extract_single(&result)?;
        Ok(DaemonConfig::deserialize(&value)?)
    }

    async fn get_config_value(&self, key: &str) -> Result<RencodeValue, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.get_config_value")
                    .with_args(vec![RencodeValue::Str(key.to_owned())]),
            )
            .await?;
        Ok(extract_single(&result)?)
    }

    async fn get_config_values(
        &self,
        keys: &[String],
    ) -> Result<BTreeMap<String, RencodeValue>, DelugeRpcError> {
        let key_values: Vec<RencodeValue> =
            keys.iter().map(|k| RencodeValue::Str(k.clone())).collect();
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.get_config_values")
                    .with_args(vec![RencodeValue::List(key_values)]),
            )
            .await?;
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
                            return Err(DelugeRpcError::UnexpectedResponseType {
                                method: "core.get_config_values returned non-str key".into(),
                                value: other,
                            });
                        }
                    }
                }
                Ok(out)
            }
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.get_config_values".into(),
                value: other,
            }),
        }
    }

    async fn set_config(
        &self,
        config: &BTreeMap<String, RencodeValue>,
    ) -> Result<(), DelugeRpcError> {
        let config_dict: BTreeMap<RencodeValue, RencodeValue> = config
            .iter()
            .map(|(k, v)| (RencodeValue::Str(k.clone()), v.clone()))
            .collect();
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.set_config")
                    .with_args(vec![RencodeValue::Dict(config_dict)]),
            )
            .await?;
        Ok(())
    }

    async fn get_proxy(&self) -> Result<ProxyConfig, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.get_proxy"))
            .await?;
        let value = extract_single(&result)?;
        Ok(ProxyConfig::deserialize(&value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
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
