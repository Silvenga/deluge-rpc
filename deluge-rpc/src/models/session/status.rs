use crate::RencodeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Session status returned by `core.get_session_status(keys)`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SessionStatus {
    pub download_rate: f64,
    pub upload_rate: f64,
    pub payload_download_rate: f64,
    pub payload_upload_rate: f64,
    pub ip_overhead_download_rate: f64,
    pub ip_overhead_upload_rate: f64,
    pub tracker_download_rate: f64,
    pub tracker_upload_rate: f64,
    pub dht_download_rate: f64,
    pub dht_upload_rate: f64,

    pub write_hit_ratio: f64,
    pub read_hit_ratio: f64,

    #[serde(flatten)]
    pub extra: HashMap<String, RencodeValue>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    fn make_session_status_dict() -> RencodeValue {
        let mut map = BTreeMap::new();

        // Rate keys
        map.insert(
            RencodeValue::Str("download_rate".into()),
            RencodeValue::Float(1024.0),
        );
        map.insert(
            RencodeValue::Str("upload_rate".into()),
            RencodeValue::Float(512.0),
        );
        map.insert(
            RencodeValue::Str("payload_download_rate".into()),
            RencodeValue::Float(1000.0),
        );
        map.insert(
            RencodeValue::Str("payload_upload_rate".into()),
            RencodeValue::Float(500.0),
        );
        map.insert(
            RencodeValue::Str("ip_overhead_download_rate".into()),
            RencodeValue::Float(10.0),
        );
        map.insert(
            RencodeValue::Str("ip_overhead_upload_rate".into()),
            RencodeValue::Float(5.0),
        );
        map.insert(
            RencodeValue::Str("tracker_download_rate".into()),
            RencodeValue::Float(0.0),
        );
        map.insert(
            RencodeValue::Str("tracker_upload_rate".into()),
            RencodeValue::Float(0.0),
        );
        map.insert(
            RencodeValue::Str("dht_download_rate".into()),
            RencodeValue::Float(14.0),
        );
        map.insert(
            RencodeValue::Str("dht_upload_rate".into()),
            RencodeValue::Float(7.0),
        );

        // Cache hit ratios
        map.insert(
            RencodeValue::Str("write_hit_ratio".into()),
            RencodeValue::Float(0.95),
        );
        map.insert(
            RencodeValue::Str("read_hit_ratio".into()),
            RencodeValue::Float(0.88),
        );

        // Some overflow keys
        map.insert(
            RencodeValue::Str("peer.num_peers_connected".into()),
            RencodeValue::Int(42),
        );
        map.insert(
            RencodeValue::Str("dht.dht_nodes".into()),
            RencodeValue::Int(256),
        );
        map.insert(
            RencodeValue::Str("net.sent_bytes".into()),
            RencodeValue::Int(1_048_576),
        );

        RencodeValue::Dict(map)
    }

    #[test]
    fn when_session_status_keys_then_metrics_parsed() {
        let value = make_session_status_dict();

        let result: SessionStatus = SessionStatus::deserialize(&value).expect("deserialize");

        assert!((result.download_rate - 1024.0).abs() < f64::EPSILON);
        assert!((result.upload_rate - 512.0).abs() < f64::EPSILON);
        assert!((result.payload_download_rate - 1000.0).abs() < f64::EPSILON);
        assert!((result.payload_upload_rate - 500.0).abs() < f64::EPSILON);
        assert!((result.ip_overhead_download_rate - 10.0).abs() < f64::EPSILON);
        assert!((result.ip_overhead_upload_rate - 5.0).abs() < f64::EPSILON);
        assert!((result.tracker_download_rate - 0.0).abs() < f64::EPSILON);
        assert!((result.tracker_upload_rate - 0.0).abs() < f64::EPSILON);
        assert!((result.dht_download_rate - 14.0).abs() < f64::EPSILON);
        assert!((result.dht_upload_rate - 7.0).abs() < f64::EPSILON);
        assert!((result.write_hit_ratio - 0.95).abs() < f64::EPSILON);
        assert!((result.read_hit_ratio - 0.88).abs() < f64::EPSILON);

        assert_eq!(result.extra.len(), 3);
        assert_eq!(
            result.extra.get("peer.num_peers_connected"),
            Some(&RencodeValue::Int(42))
        );
        assert_eq!(
            result.extra.get("dht.dht_nodes"),
            Some(&RencodeValue::Int(256))
        );
        assert_eq!(
            result.extra.get("net.sent_bytes"),
            Some(&RencodeValue::Int(1_048_576))
        );
    }

    #[test]
    fn when_session_status_extra_key_deserialized_then_typed_access_works() {
        let value = make_session_status_dict();

        let result: SessionStatus = SessionStatus::deserialize(&value).expect("deserialize");

        let peer_count: i64 = i64::deserialize(&result.extra["peer.num_peers_connected"])
            .expect("deserialize peer count");
        assert_eq!(peer_count, 42);
    }
}
