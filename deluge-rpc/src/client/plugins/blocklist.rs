use crate::client::RpcCaller;
use crate::models::plugins::{BlocklistConfig, BlocklistStatus};
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use crate::rencode::{RencodeValue, to_rencode_value};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use serde::Deserialize;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait BlocklistRpc: Send + Sync {
    async fn check_import(&self, force: bool) -> anyhow::Result<Option<String>>;
    async fn get_config(&self) -> anyhow::Result<BlocklistConfig>;
    async fn set_config(&self, config: &BlocklistConfig) -> anyhow::Result<()>;
    async fn get_status(&self) -> anyhow::Result<BlocklistStatus>;
}

pub struct BlocklistClient {
    caller: RpcCaller,
}

impl BlocklistClient {
    pub(crate) fn new(caller: RpcCaller) -> Self {
        Self { caller }
    }
}

impl Clone for BlocklistClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl BlocklistRpc for BlocklistClient {
    async fn check_import(&self, force: bool) -> anyhow::Result<Option<String>> {
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("blocklist.check_import")
                    .with_args(vec![RencodeValue::Bool(force)]),
            )
            .await
            .context("blocklist.check_import RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::None => Ok(None),
            RencodeValue::Str(s) => Ok(Some(s)),
            other => Err(anyhow!(
                "blocklist.check_import returned unexpected type: {other:?}"
            )),
        }
    }

    async fn get_config(&self) -> anyhow::Result<BlocklistConfig> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("blocklist.get_config"))
            .await
            .context("blocklist.get_config RPC failed")?;
        let value = extract_single(&result)?;
        BlocklistConfig::deserialize(&value).context("deserializing blocklist config")
    }

    async fn set_config(&self, config: &BlocklistConfig) -> anyhow::Result<()> {
        let config_value = to_rencode_value(config).context("serializing blocklist config")?;
        self.caller
            .rpc_call(DelugeRpcRequest::new("blocklist.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }

    async fn get_status(&self) -> anyhow::Result<BlocklistStatus> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("blocklist.get_status"))
            .await
            .context("blocklist.get_status RPC failed")?;
        let value = extract_single(&result)?;
        BlocklistStatus::deserialize(&value).context("deserializing blocklist status")
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
    fn when_blocklist_get_status_response_then_deserializes() {
        let response = make_dict(vec![
            ("state", RencodeValue::Str("Idle".into())),
            ("up_to_date", RencodeValue::Bool(true)),
            ("num_whited", RencodeValue::Int(10)),
            ("num_blocked", RencodeValue::Int(5000)),
            ("file_progress", RencodeValue::Float(1.0)),
            (
                "file_url",
                RencodeValue::Str("https://example.com/blocklist.txt".into()),
            ),
            ("file_size", RencodeValue::Int(1_048_576)),
            ("file_date", RencodeValue::Float(1_700_000_000.0)),
            ("file_type", RencodeValue::Str("p2p (gz)".into())),
            (
                "whitelisted",
                RencodeValue::List(vec![RencodeValue::Str("10.0.0.1".into())]),
            ),
        ]);

        let value = extract_single(&response).expect("extract");
        let status: BlocklistStatus = BlocklistStatus::deserialize(&value).expect("deserialize");

        assert_eq!(status.state, "Idle");
        assert!(status.up_to_date);
        assert_eq!(status.num_blocked, 5000);
    }
}
