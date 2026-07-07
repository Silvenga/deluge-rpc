use serde::{Deserialize, Serialize};

/// Connected peer, from the `peers` sub-dict of `TorrentStatus`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PeerInfo {
    /// Peer client.
    pub client: String,
    /// Peer country.
    pub country: String,
    /// Peer download speed.
    pub down_speed: i64,
    /// Peer IP address.
    pub ip: String,
    /// Peer progress.
    pub progress: f64,
    /// Whether the peer is a seed.
    pub seed: bool,
    /// Peer upload speed.
    pub up_speed: i64,
}

/// File entry, from the `files` / `orig_files` sub-dict of `TorrentStatus`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FileInfo {
    /// File index.
    pub index: i64,
    /// File path.
    pub path: String,
    /// File size.
    pub size: i64,
    /// File offset.
    pub offset: i64,
}

/// Tracker entry, from the `trackers` sub-dict of `TorrentStatus`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TrackerInfo {
    /// Tracker URL.
    pub url: String,
    /// Tracker tier.
    pub tier: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rencode::RencodeValue;
    use std::collections::BTreeMap;

    fn make_peer_dict() -> RencodeValue {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("client".into()),
            RencodeValue::Str("qBittorrent 4.5.0".into()),
        );
        map.insert(
            RencodeValue::Str("country".into()),
            RencodeValue::Str("US".into()),
        );
        map.insert(
            RencodeValue::Str("down_speed".into()),
            RencodeValue::Int(102400),
        );
        map.insert(
            RencodeValue::Str("ip".into()),
            RencodeValue::Str("192.168.1.100".into()),
        );
        map.insert(
            RencodeValue::Str("progress".into()),
            RencodeValue::Float(0.75),
        );
        map.insert(RencodeValue::Str("seed".into()), RencodeValue::Bool(false));
        map.insert(
            RencodeValue::Str("up_speed".into()),
            RencodeValue::Int(51200),
        );
        RencodeValue::Dict(map)
    }

    fn make_file_dict() -> RencodeValue {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("index".into()), RencodeValue::Int(0));
        map.insert(
            RencodeValue::Str("path".into()),
            RencodeValue::Str("video.mkv".into()),
        );
        map.insert(
            RencodeValue::Str("size".into()),
            RencodeValue::Int(1_073_741_824),
        );
        map.insert(RencodeValue::Str("offset".into()), RencodeValue::Int(0));
        RencodeValue::Dict(map)
    }

    fn make_tracker_dict() -> RencodeValue {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("url".into()),
            RencodeValue::Str("http://tracker.example.com:6969/announce".into()),
        );
        map.insert(RencodeValue::Str("tier".into()), RencodeValue::Int(0));
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_peer_dict_then_all_fields_populate() {
        let value = make_peer_dict();

        let result: PeerInfo = PeerInfo::deserialize(&value).expect("deserialize");

        assert_eq!(result.client, "qBittorrent 4.5.0");
        assert_eq!(result.country, "US");
        assert_eq!(result.down_speed, 102400);
        assert_eq!(result.ip, "192.168.1.100");
        assert!((result.progress - 0.75).abs() < f64::EPSILON);
        assert!(!result.seed);
        assert_eq!(result.up_speed, 51200);
    }

    #[test]
    fn when_file_dict_then_all_fields_populate() {
        let value = make_file_dict();

        let result: FileInfo = FileInfo::deserialize(&value).expect("deserialize");

        assert_eq!(result.index, 0);
        assert_eq!(result.path, "video.mkv");
        assert_eq!(result.size, 1_073_741_824);
        assert_eq!(result.offset, 0);
    }

    #[test]
    fn when_tracker_dict_then_all_fields_populate() {
        let value = make_tracker_dict();

        let result: TrackerInfo = TrackerInfo::deserialize(&value).expect("deserialize");

        assert_eq!(result.url, "http://tracker.example.com:6969/announce");
        assert_eq!(result.tier, 0);
    }

    #[test]
    fn when_peer_list_then_deserialized_as_vec() {
        let list = RencodeValue::List(vec![make_peer_dict(), make_peer_dict()]);

        let result: Vec<PeerInfo> = Vec::deserialize(&list).expect("deserialize");

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn when_file_list_then_deserialized_as_vec() {
        let list = RencodeValue::List(vec![make_file_dict(), make_file_dict()]);

        let result: Vec<FileInfo> = Vec::deserialize(&list).expect("deserialize");

        assert_eq!(result.len(), 2);
    }
}
