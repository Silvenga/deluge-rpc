use crate::DelugeRpcError;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::ExtractorConfig;
use crate::protocol::{DelugeRpcRequest, extract_single};
use crate::to_rencode_value;
use async_trait::async_trait;
use serde::Deserialize;

#[async_trait]
pub trait ExtractorRpc: Send + Sync {
    async fn set_config(&self, config: &ExtractorConfig) -> Result<(), DelugeRpcError>;
    async fn get_config(&self) -> Result<ExtractorConfig, DelugeRpcError>;
}

pub struct ExtractorClient {
    dispatcher: DelugeClientDispatcher,
}

impl ExtractorClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for ExtractorClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl ExtractorRpc for ExtractorClient {
    async fn set_config(&self, config: &ExtractorConfig) -> Result<(), DelugeRpcError> {
        let config_value = to_rencode_value(config)?;
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("extractor.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> Result<ExtractorConfig, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("extractor.get_config"))
            .await?;
        let value = extract_single(&result)?;
        Ok(ExtractorConfig::deserialize(&value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
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
