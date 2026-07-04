use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the Stats plugin.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StatsConfig {
    /// Test key (unused).
    pub test: String,
    /// Sampling interval in seconds.
    pub update_interval: i64,
    /// Number of samples to keep.
    pub length: i64,
}

/// Cumulative totals returned by `stats.get_totals` and `stats.get_session_totals`.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct StatsTotals {
    /// Total bytes uploaded.
    pub total_upload: i64,
    /// Total bytes downloaded.
    pub total_download: i64,
    /// Total payload bytes uploaded.
    pub total_payload_upload: i64,
    /// Total payload bytes downloaded.
    pub total_payload_download: i64,
}

/// Return value of `stats.get_stats`.
///
/// Contains historical stat samples keyed by stat name, plus metadata fields.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct StatsGetStatsResult {
    /// Unix timestamp of the last update.
    #[serde(rename = "_last_update")]
    pub last_update: f64,
    /// Number of samples stored.
    #[serde(rename = "_length")]
    pub length: i64,
    /// Sampling interval in seconds.
    #[serde(rename = "_update_interval")]
    pub update_interval: i64,
    /// Historical stat samples keyed by stat name (e.g. "upload_rate", "download_rate").
    #[serde(flatten)]
    pub stats: HashMap<String, Vec<i64>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use std::collections::BTreeMap;

    fn make_dict(entries: Vec<(&str, RencodeValue)>) -> RencodeValue {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(RencodeValue::Str(k.into()), v);
        }
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_stats_config_dict_then_fields_populate() {
        let value = make_dict(vec![
            ("test", RencodeValue::Str("NiNiNi".into())),
            ("update_interval", RencodeValue::Int(1)),
            ("length", RencodeValue::Int(150)),
        ]);

        let result: StatsConfig = StatsConfig::deserialize(&value).expect("deserialize");

        assert_eq!(result.test, "NiNiNi");
        assert_eq!(result.update_interval, 1);
        assert_eq!(result.length, 150);
    }

    #[test]
    fn when_stats_totals_dict_then_fields_populate() {
        let value = make_dict(vec![
            ("total_upload", RencodeValue::Int(1_000_000)),
            ("total_download", RencodeValue::Int(5_000_000)),
            ("total_payload_upload", RencodeValue::Int(900_000)),
            ("total_payload_download", RencodeValue::Int(4_500_000)),
        ]);

        let result: StatsTotals = StatsTotals::deserialize(&value).expect("deserialize");

        assert_eq!(result.total_upload, 1_000_000);
        assert_eq!(result.total_download, 5_000_000);
        assert_eq!(result.total_payload_upload, 900_000);
        assert_eq!(result.total_payload_download, 4_500_000);
    }

    #[test]
    fn when_stats_get_stats_result_dict_then_fields_populate() {
        let value = make_dict(vec![
            ("_last_update", RencodeValue::Float(1_700_000_000.0)),
            ("_length", RencodeValue::Int(150)),
            ("_update_interval", RencodeValue::Int(5)),
            (
                "upload_rate",
                RencodeValue::List(vec![RencodeValue::Int(100), RencodeValue::Int(200)]),
            ),
            (
                "download_rate",
                RencodeValue::List(vec![RencodeValue::Int(500), RencodeValue::Int(1000)]),
            ),
        ]);

        let result: StatsGetStatsResult =
            StatsGetStatsResult::deserialize(&value).expect("deserialize");

        assert!((result.last_update - 1_700_000_000.0).abs() < f64::EPSILON);
        assert_eq!(result.length, 150);
        assert_eq!(result.update_interval, 5);
        assert_eq!(
            result.stats.get("upload_rate").map(|v| v.as_slice()),
            Some(&[100i64, 200][..])
        );
        assert_eq!(
            result.stats.get("download_rate").map(|v| v.as_slice()),
            Some(&[500i64, 1000][..])
        );
    }
}
