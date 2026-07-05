use crate::client::RpcCaller;
use crate::models::plugins::ExtractorConfig;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use crate::rencode::to_rencode_value;
use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait ExtractorRpc: Send + Sync {
    async fn set_config(&self, config: &ExtractorConfig) -> anyhow::Result<()>;
    async fn get_config(&self) -> anyhow::Result<ExtractorConfig>;
}

pub struct ExtractorClient {
    caller: RpcCaller,
}

impl ExtractorClient {
    pub(crate) fn new(caller: RpcCaller) -> Self {
        Self { caller }
    }
}

impl Clone for ExtractorClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl ExtractorRpc for ExtractorClient {
    async fn set_config(&self, config: &ExtractorConfig) -> anyhow::Result<()> {
        let config_value = to_rencode_value(config).context("serializing extractor config")?;
        self.caller
            .rpc_call(DelugeRpcRequest::new("extractor.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> anyhow::Result<ExtractorConfig> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("extractor.get_config"))
            .await
            .context("extractor.get_config RPC failed")?;
        let value = extract_single(&result)?;
        ExtractorConfig::deserialize(&value).context("deserializing extractor config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    fn make_dict(entries: Vec<(&str, RencodeValue)>) -> RencodeValue {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(RencodeValue::Str(k.into()), v);
        }
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_extractor_get_config_response_then_deserializes() {
        let response = make_dict(vec![
            ("extract_path", RencodeValue::Str("/tmp/extract".into())),
            ("use_name_folder", RencodeValue::Bool(false)),
        ]);

        let value = extract_single(&response).expect("extract");
        let config: ExtractorConfig = ExtractorConfig::deserialize(&value).expect("deserialize");

        assert_eq!(config.extract_path, "/tmp/extract");
        assert!(!config.use_name_folder);
    }
}
