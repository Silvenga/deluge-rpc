use crate::client::caller::RpcCaller;
use crate::models::{LabelConfig, LabelOptions};
use crate::protocol::{extract_single, DelugeRpcRequest};
use crate::rencode::{to_rencode_value, RencodeValue};
use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait LabelRpc: Send + Sync {
    async fn get_labels(&self) -> anyhow::Result<Vec<String>>;
    async fn add(&self, label_id: &str) -> anyhow::Result<()>;
    async fn remove(&self, label_id: &str) -> anyhow::Result<()>;
    async fn set_options(&self, label_id: &str, options: &LabelOptions) -> anyhow::Result<()>;
    async fn get_options(&self, label_id: &str) -> anyhow::Result<LabelOptions>;
    async fn set_torrent(&self, torrent_id: &str, label_id: &str) -> anyhow::Result<()>;
    async fn get_config(&self) -> anyhow::Result<LabelConfig>;
    async fn set_config(&self, config: &LabelConfig) -> anyhow::Result<()>;
}

pub struct LabelClient {
    caller: RpcCaller,
}

impl LabelClient {
    pub(crate) fn new(caller: RpcCaller) -> Self {
        Self { caller }
    }
}

impl Clone for LabelClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl LabelRpc for LabelClient {
    async fn get_labels(&self) -> anyhow::Result<Vec<String>> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("label.get_labels"))
            .await
            .context("label.get_labels RPC failed")?;
        let value = extract_single(&result)?;
        Vec::<String>::deserialize(&value).context("deserializing labels")
    }

    async fn add(&self, label_id: &str) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("label.add")
                    .with_args(vec![RencodeValue::Str(label_id.to_owned())]),
            )
            .await?;
        Ok(())
    }

    async fn remove(&self, label_id: &str) -> anyhow::Result<()> {
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("label.remove")
                    .with_args(vec![RencodeValue::Str(label_id.to_owned())]),
            )
            .await?;
        Ok(())
    }

    async fn set_options(&self, label_id: &str, options: &LabelOptions) -> anyhow::Result<()> {
        let options_value = to_rencode_value(options).context("serializing label options")?;
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("label.set_options")
                    .with_args(vec![RencodeValue::Str(label_id.to_owned()), options_value]),
            )
            .await?;
        Ok(())
    }

    async fn get_options(&self, label_id: &str) -> anyhow::Result<LabelOptions> {
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("label.get_options")
                    .with_args(vec![RencodeValue::Str(label_id.to_owned())]),
            )
            .await
            .context("label.get_options RPC failed")?;
        let value = extract_single(&result)?;
        LabelOptions::deserialize(&value).context("deserializing label options")
    }

    async fn set_torrent(&self, torrent_id: &str, label_id: &str) -> anyhow::Result<()> {
        self.caller
            .rpc_call(DelugeRpcRequest::new("label.set_torrent").with_args(vec![
                RencodeValue::Str(torrent_id.to_owned()),
                RencodeValue::Str(label_id.to_owned()),
            ]))
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> anyhow::Result<LabelConfig> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("label.get_config"))
            .await
            .context("label.get_config RPC failed")?;
        let value = extract_single(&result)?;
        LabelConfig::deserialize(&value).context("deserializing label config")
    }

    async fn set_config(&self, config: &LabelConfig) -> anyhow::Result<()> {
        let config_value = to_rencode_value(config).context("serializing label config")?;
        self.caller
            .rpc_call(DelugeRpcRequest::new("label.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;
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
