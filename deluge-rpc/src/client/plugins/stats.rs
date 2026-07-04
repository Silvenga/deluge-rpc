use crate::client::RpcCaller;
use crate::models::plugins::{StatsConfig, StatsGetStatsResult, StatsTotals};
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use crate::rencode::{RencodeValue, to_rencode_value};
use crate::shared::Shared;
use crate::transport::DelugeWriter;
use anyhow::Context;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Deserialize;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait StatsRpc: Send + Sync {
    async fn get_stats(&self, keys: &[String], interval: i64) -> anyhow::Result<Option<StatsGetStatsResult>>;
    async fn get_totals(&self) -> anyhow::Result<StatsTotals>;
    async fn get_session_totals(&self) -> anyhow::Result<StatsTotals>;
    async fn set_config(&self, config: &StatsConfig) -> anyhow::Result<()>;
    async fn get_config(&self) -> anyhow::Result<StatsConfig>;
    async fn get_intervals(&self) -> anyhow::Result<Vec<i64>>;
}

pub struct StatsClient {
    caller: RpcCaller,
}

impl StatsClient {
    pub(crate) fn new(shared: Arc<Shared>, writer: Arc<Mutex<DelugeWriter>>) -> Self {
        Self {
            caller: RpcCaller::new(shared, writer),
        }
    }
}

impl Clone for StatsClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl StatsRpc for StatsClient {
    async fn get_stats(&self, keys: &[String], interval: i64) -> anyhow::Result<Option<StatsGetStatsResult>> {
        let keys_list: Vec<RencodeValue> = keys.iter().map(|k| RencodeValue::Str(k.clone())).collect();
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("stats.get_stats").with_args(vec![
                    RencodeValue::List(keys_list),
                    RencodeValue::Int(interval),
                ]),
            )
            .await
            .context("stats.get_stats RPC failed")?;
        let value = extract_single(&result, "stats.get_stats")?;
        match value {
            RencodeValue::None => Ok(None),
            other => StatsGetStatsResult::deserialize(&other)
                .map(Some)
                .context("deserializing stats result"),
        }
    }

    async fn get_totals(&self) -> anyhow::Result<StatsTotals> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("stats.get_totals"))
            .await
            .context("stats.get_totals RPC failed")?;
        let value = extract_single(&result, "stats.get_totals")?;
        StatsTotals::deserialize(&value).context("deserializing stats totals")
    }

    async fn get_session_totals(&self) -> anyhow::Result<StatsTotals> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("stats.get_session_totals"))
            .await
            .context("stats.get_session_totals RPC failed")?;
        let value = extract_single(&result, "stats.get_session_totals")?;
        StatsTotals::deserialize(&value).context("deserializing session totals")
    }

    async fn set_config(&self, config: &StatsConfig) -> anyhow::Result<()> {
        let config_value = to_rencode_value(config).context("serializing stats config")?;
        self.caller
            .rpc_call(
                DelugeRpcRequest::new("stats.set_config").with_args(vec![config_value]),
            )
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> anyhow::Result<StatsConfig> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("stats.get_config"))
            .await
            .context("stats.get_config RPC failed")?;
        let value = extract_single(&result, "stats.get_config")?;
        StatsConfig::deserialize(&value).context("deserializing stats config")
    }

    async fn get_intervals(&self) -> anyhow::Result<Vec<i64>> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("stats.get_intervals"))
            .await
            .context("stats.get_intervals RPC failed")?;
        let value = extract_single(&result, "stats.get_intervals")?;
        Vec::<i64>::deserialize(&value).context("deserializing intervals")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_stats_get_intervals_response_then_deserializes() {
        let response = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Int(1),
            RencodeValue::Int(5),
            RencodeValue::Int(30),
            RencodeValue::Int(300),
        ])]);

        let value = extract_single(&response, "stats.get_intervals").expect("extract");
        let intervals: Vec<i64> = Vec::<i64>::deserialize(&value).expect("deserialize");

        assert_eq!(intervals, vec![1, 5, 30, 300]);
    }
}
