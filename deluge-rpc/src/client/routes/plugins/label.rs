use crate::DelugeRpcError;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{LabelConfig, LabelOptions};
use crate::protocol::{DelugeRpcRequest, extract_single};
use crate::{RencodeValue, to_rencode_value};
use async_trait::async_trait;
use serde::Deserialize;

#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait LabelRpc: Send + Sync {
    async fn get_labels(&self) -> Result<Vec<String>, DelugeRpcError>;
    async fn add(&self, label_id: &str) -> Result<(), DelugeRpcError>;
    async fn remove(&self, label_id: &str) -> Result<(), DelugeRpcError>;
    async fn set_options(
        &self,
        label_id: &str,
        options: &LabelOptions,
    ) -> Result<(), DelugeRpcError>;
    async fn get_options(&self, label_id: &str) -> Result<LabelOptions, DelugeRpcError>;
    async fn set_torrent(&self, torrent_id: &str, label_id: &str) -> Result<(), DelugeRpcError>;
    async fn get_config(&self) -> Result<LabelConfig, DelugeRpcError>;
    async fn set_config(&self, config: &LabelConfig) -> Result<(), DelugeRpcError>;
}

pub struct LabelClient {
    dispatcher: DelugeClientDispatcher,
}

impl LabelClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for LabelClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl LabelRpc for LabelClient {
    async fn get_labels(&self) -> Result<Vec<String>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("label.get_labels"))
            .await?;
        let value = extract_single(&result)?;
        Ok(Vec::<String>::deserialize(&value)?)
    }

    async fn add(&self, label_id: &str) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("label.add")
                    .with_args(vec![RencodeValue::Str(label_id.to_owned())]),
            )
            .await?;
        Ok(())
    }

    async fn remove(&self, label_id: &str) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("label.remove")
                    .with_args(vec![RencodeValue::Str(label_id.to_owned())]),
            )
            .await?;
        Ok(())
    }

    async fn set_options(
        &self,
        label_id: &str,
        options: &LabelOptions,
    ) -> Result<(), DelugeRpcError> {
        let options_value = to_rencode_value(options)?;
        self.dispatcher
            .dispatch(
                DelugeRpcRequest::new("label.set_options")
                    .with_args(vec![RencodeValue::Str(label_id.to_owned()), options_value]),
            )
            .await?;
        Ok(())
    }

    async fn get_options(&self, label_id: &str) -> Result<LabelOptions, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("label.get_options")
                    .with_args(vec![RencodeValue::Str(label_id.to_owned())]),
            )
            .await?;
        let value = extract_single(&result)?;
        Ok(LabelOptions::deserialize(&value)?)
    }

    async fn set_torrent(&self, torrent_id: &str, label_id: &str) -> Result<(), DelugeRpcError> {
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("label.set_torrent").with_args(vec![
                RencodeValue::Str(torrent_id.to_owned()),
                RencodeValue::Str(label_id.to_owned()),
            ]))
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> Result<LabelConfig, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("label.get_config"))
            .await?;
        let value = extract_single(&result)?;
        Ok(LabelConfig::deserialize(&value)?)
    }

    async fn set_config(&self, config: &LabelConfig) -> Result<(), DelugeRpcError> {
        let config_value = to_rencode_value(config)?;
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("label.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_label_get_labels_response_then_deserializes() {
        let response = RencodeValue::List(vec![
            RencodeValue::Str("movies".into()),
            RencodeValue::Str("music".into()),
        ]);

        let value = extract_single(&response).expect("extract");
        let labels: Vec<String> = Vec::<String>::deserialize(&value).expect("deserialize");

        assert_eq!(labels, vec!["movies", "music"]);
    }
}
