use crate::DelugeRpcError;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{BlocklistConfig, BlocklistStatus};
use crate::protocol::{DelugeRpcRequest, extract_single};
use crate::{RencodeValue, to_rencode_value};

use serde::Deserialize;

/// RPC methods for the `blocklist.*` namespace.
pub trait BlocklistRpc: Send + Sync {
    /// Downloads and imports the blocklist from the configured URL.
    async fn check_import(&self, force: bool) -> Result<Option<String>, DelugeRpcError>;
    /// Returns the plugin config.
    async fn get_config(&self) -> Result<BlocklistConfig, DelugeRpcError>;
    /// Sets the plugin config. May trigger a re-import if the URL changed.
    async fn set_config(&self, config: &BlocklistConfig) -> Result<(), DelugeRpcError>;
    /// Returns the current import status.
    async fn get_status(&self) -> Result<BlocklistStatus, DelugeRpcError>;
}

/// Client for `blocklist.*` RPC methods.
pub struct BlocklistClient {
    dispatcher: DelugeClientDispatcher,
}

impl BlocklistClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for BlocklistClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

impl BlocklistRpc for BlocklistClient {
    async fn check_import(&self, force: bool) -> Result<Option<String>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("blocklist.check_import")
                    .with_args(vec![RencodeValue::Bool(force)]),
            )
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::None => Ok(None),
            RencodeValue::Str(s) => Ok(Some(s)),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "blocklist.check_import returned unexpected type".into(),
                value: other,
            }),
        }
    }

    async fn get_config(&self) -> Result<BlocklistConfig, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("blocklist.get_config"))
            .await?;
        let value = extract_single(&result)?;
        Ok(BlocklistConfig::deserialize(&value)?)
    }

    async fn set_config(&self, config: &BlocklistConfig) -> Result<(), DelugeRpcError> {
        let config_value = to_rencode_value(config)?;
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("blocklist.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }

    async fn get_status(&self) -> Result<BlocklistStatus, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("blocklist.get_status"))
            .await?;
        let value = extract_single(&result)?;
        Ok(BlocklistStatus::deserialize(&value)?)
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
