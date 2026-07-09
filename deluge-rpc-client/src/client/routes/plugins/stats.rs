use crate::DelugeRpcError;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{StatsConfig, StatsGetStatsResult, StatsTotals};
use crate::protocol::{DelugeRpcRequest, extract_single};
use crate::{RencodeValue, to_rencode_value};
use async_trait::async_trait;
use serde::Deserialize;

/// RPC methods for the `stats.*` namespace.
#[async_trait]
pub trait StatsRpc: Send + Sync {
    /// Returns historical stats for the requested keys at the given interval.
    async fn get_stats(
        &self,
        keys: &[String],
        interval: i64,
    ) -> Result<Option<StatsGetStatsResult>, DelugeRpcError>;
    /// Returns cumulative totals (persisted + current session).
    async fn get_totals(&self) -> Result<StatsTotals, DelugeRpcError>;
    /// Returns current session totals.
    async fn get_session_totals(&self) -> Result<StatsTotals, DelugeRpcError>;
    /// Sets the plugin config.
    async fn set_config(&self, config: &StatsConfig) -> Result<(), DelugeRpcError>;
    /// Returns the plugin config.
    async fn get_config(&self) -> Result<StatsConfig, DelugeRpcError>;
    /// Returns valid sampling intervals.
    async fn get_intervals(&self) -> Result<Vec<i64>, DelugeRpcError>;
}

/// Client for `stats.*` RPC methods.
pub struct StatsClient {
    dispatcher: DelugeClientDispatcher,
}

impl StatsClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for StatsClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl StatsRpc for StatsClient {
    async fn get_stats(
        &self,
        keys: &[String],
        interval: i64,
    ) -> Result<Option<StatsGetStatsResult>, DelugeRpcError> {
        let keys_list: Vec<RencodeValue> =
            keys.iter().map(|k| RencodeValue::Str(k.clone())).collect();
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("stats.get_stats").with_args(vec![
                RencodeValue::List(keys_list),
                RencodeValue::Int(interval),
            ]))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::None => Ok(None),
            other => Ok(Some(StatsGetStatsResult::deserialize(&other)?)),
        }
    }

    async fn get_totals(&self) -> Result<StatsTotals, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("stats.get_totals"))
            .await?;
        let value = extract_single(&result)?;
        Ok(StatsTotals::deserialize(&value)?)
    }

    async fn get_session_totals(&self) -> Result<StatsTotals, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("stats.get_session_totals"))
            .await?;
        let value = extract_single(&result)?;
        Ok(StatsTotals::deserialize(&value)?)
    }

    async fn set_config(&self, config: &StatsConfig) -> Result<(), DelugeRpcError> {
        let config_value = to_rencode_value(config)?;
        self.dispatcher
            .dispatch(DelugeRpcRequest::new("stats.set_config").with_args(vec![config_value]))
            .await?;
        Ok(())
    }

    async fn get_config(&self) -> Result<StatsConfig, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("stats.get_config"))
            .await?;
        let value = extract_single(&result)?;
        Ok(StatsConfig::deserialize(&value)?)
    }

    async fn get_intervals(&self) -> Result<Vec<i64>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("stats.get_intervals"))
            .await?;
        let value = extract_single(&result)?;
        Ok(Vec::<i64>::deserialize(&value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_stats_get_intervals_response_then_deserializes() {
        let response = RencodeValue::List(vec![
            RencodeValue::Int(1),
            RencodeValue::Int(5),
            RencodeValue::Int(30),
            RencodeValue::Int(300),
        ]);

        let value = extract_single(&response).expect("extract");
        let intervals: Vec<i64> = Vec::<i64>::deserialize(&value).expect("deserialize");

        assert_eq!(intervals, vec![1, 5, 30, 300]);
    }
}
