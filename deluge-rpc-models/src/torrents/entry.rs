use super::status::TorrentStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TorrentEntry {
    #[serde(skip)]
    pub info_hash: String,
    #[serde(flatten)]
    pub status: TorrentStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rencode::RencodeValue;
    use std::collections::BTreeMap;

    fn make_entry_dict() -> RencodeValue {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("name".into()),
            RencodeValue::Str("test-torrent".into()),
        );
        map.insert(
            RencodeValue::Str("state".into()),
            RencodeValue::Str("Seeding".into()),
        );
        map.insert(
            RencodeValue::Str("progress".into()),
            RencodeValue::Float(100.0),
        );
        map.insert(RencodeValue::Str("ratio".into()), RencodeValue::Float(2.5));
        map.insert(
            RencodeValue::Str("hash".into()),
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
        );
        map.insert(RencodeValue::Str("eta".into()), RencodeValue::Int(-1));
        map.insert(
            RencodeValue::Str("completed_time".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("last_seen_complete".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("time_since_download".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("time_since_transfer".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("time_since_upload".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("seeds_peers_ratio".into()),
            RencodeValue::Float(-1.0),
        );
        map.insert(
            RencodeValue::Str("max_connections".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_download_speed".into()),
            RencodeValue::Float(-1.0),
        );
        map.insert(
            RencodeValue::Str("max_upload_slots".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_upload_speed".into()),
            RencodeValue::Float(-1.0),
        );
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_torrent_entry_then_flatten_works() {
        let value = make_entry_dict();

        let mut entry = TorrentEntry::deserialize(&value).expect("deserialize");
        entry.info_hash = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into();

        assert_eq!(entry.info_hash, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
        assert_eq!(entry.status.name, "test-torrent");
        assert_eq!(entry.status.state, "Seeding");
        assert!((entry.status.progress - 100.0).abs() < f64::EPSILON);
        assert!((entry.status.ratio.unwrap() - 2.5).abs() < f64::EPSILON);
        assert_eq!(entry.status.eta, None);
        assert_eq!(entry.status.completed_time, None);
        assert_eq!(entry.status.last_seen_complete, None);
        assert_eq!(entry.status.time_since_download, None);
        assert_eq!(entry.status.time_since_transfer, None);
        assert_eq!(entry.status.time_since_upload, None);
        assert_eq!(entry.status.seeds_peers_ratio, None);
        assert_eq!(entry.status.max_connections, None);
        assert_eq!(entry.status.max_download_speed, None);
        assert_eq!(entry.status.max_upload_slots, None);
        assert_eq!(entry.status.max_upload_speed, None);
    }

    #[test]
    fn when_torrent_entry_new_then_fields_set() {
        let mut entry = TorrentEntry::deserialize(&make_entry_dict()).expect("deserialize");
        entry.info_hash = "deadbeef".into();

        assert_eq!(entry.info_hash, "deadbeef");
        assert_eq!(entry.status.name, "test-torrent");
    }
}
