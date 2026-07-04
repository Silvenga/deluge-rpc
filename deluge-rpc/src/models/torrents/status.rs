use super::sub_dicts::{FileInfo, PeerInfo, TrackerInfo};
use crate::models::sentinels::{
    deserialize_never_i64, deserialize_ratio, deserialize_unlimited_f64, deserialize_unlimited_i64,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct TorrentStatus {
    // --- Time / transfer stats ---
    pub active_time: i64,
    pub all_time_download: i64,
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub completed_time: Option<i64>,
    pub finished_time: i64,
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub last_seen_complete: Option<i64>,
    pub seeding_time: i64,
    pub time_added: i64,
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub time_since_download: Option<i64>,
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub time_since_transfer: Option<i64>,
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub time_since_upload: Option<i64>,
    pub total_done: i64,
    pub total_payload_download: i64,
    pub total_payload_upload: i64,
    pub total_remaining: i64,
    pub total_uploaded: i64,
    pub total_wanted: i64,

    // --- Rates ---
    pub download_payload_rate: i64,
    pub upload_payload_rate: i64,
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub eta: Option<i64>,

    // --- Ratios / seed metrics ---
    pub distributed_copies: f64,
    #[serde(deserialize_with = "deserialize_ratio", default)]
    pub ratio: Option<f64>,
    pub seed_rank: i64,
    #[serde(deserialize_with = "deserialize_ratio", default)]
    pub seeds_peers_ratio: Option<f64>,

    // --- Peers / seeds ---
    pub num_peers: i64,
    pub num_seeds: i64,
    pub total_peers: i64,
    pub total_seeds: i64,
    #[serde(default)]
    pub peers: Vec<PeerInfo>,

    // --- State ---
    pub state: String,
    pub paused: bool,
    pub progress: f64,
    pub is_seed: bool,
    pub is_finished: bool,
    pub seed_mode: bool,
    pub super_seeding: bool,
    #[serde(default)]
    pub message: String,
    pub queue: i64,
    pub storage_mode: String,

    // --- Identity / metadata ---
    pub hash: String,
    pub name: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub creator: String,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub num_files: i64,
    #[serde(default)]
    pub num_pieces: i64,
    #[serde(default)]
    pub piece_length: i64,
    #[serde(default)]
    pub total_size: i64,
    #[serde(default)]
    pub files: Vec<FileInfo>,
    #[serde(default)]
    pub orig_files: Vec<FileInfo>,
    #[serde(default)]
    pub pieces: Option<Vec<i64>>,

    // --- File state ---
    #[serde(default)]
    pub file_priorities: Vec<i64>,
    #[serde(default)]
    pub file_progress: Vec<f64>,

    // --- Trackers ---
    #[serde(default)]
    pub tracker: String,
    #[serde(default)]
    pub tracker_host: String,
    #[serde(default)]
    pub tracker_status: String,
    #[serde(default)]
    pub trackers: Vec<TrackerInfo>,

    // --- Options (per-torrent, returned in status) ---
    pub auto_managed: bool,
    pub is_auto_managed: bool,
    pub download_location: String,
    #[serde(deserialize_with = "deserialize_unlimited_i64", default)]
    pub max_connections: Option<i64>,
    #[serde(deserialize_with = "deserialize_unlimited_f64", default)]
    pub max_download_speed: Option<f64>,
    #[serde(deserialize_with = "deserialize_unlimited_i64", default)]
    pub max_upload_slots: Option<i64>,
    #[serde(deserialize_with = "deserialize_unlimited_f64", default)]
    pub max_upload_speed: Option<f64>,
    pub move_completed: bool,
    #[serde(default)]
    pub move_completed_path: String,
    #[serde(default)]
    pub owner: String,
    pub prioritize_first_last_pieces: bool,
    pub remove_at_ratio: bool,
    pub sequential_download: bool,
    pub shared: bool,
    pub stop_at_ratio: bool,
    pub stop_ratio: f64,
}

impl Default for TorrentStatus {
    fn default() -> Self {
        Self {
            active_time: 0,
            all_time_download: 0,
            completed_time: None,
            finished_time: 0,
            last_seen_complete: None,
            seeding_time: 0,
            time_added: 0,
            time_since_download: None,
            time_since_transfer: None,
            time_since_upload: None,
            total_done: 0,
            total_payload_download: 0,
            total_payload_upload: 0,
            total_remaining: 0,
            total_uploaded: 0,
            total_wanted: 0,
            download_payload_rate: 0,
            upload_payload_rate: 0,
            eta: None,
            distributed_copies: 0.0,
            ratio: None,
            seed_rank: 0,
            seeds_peers_ratio: None,
            num_peers: 0,
            num_seeds: 0,
            total_peers: 0,
            total_seeds: 0,
            peers: Vec::new(),
            state: String::new(),
            paused: false,
            progress: 0.0,
            is_seed: false,
            is_finished: false,
            seed_mode: false,
            super_seeding: false,
            message: String::new(),
            queue: 0,
            storage_mode: String::new(),
            hash: String::new(),
            name: String::new(),
            comment: String::new(),
            creator: String::new(),
            private: false,
            num_files: 0,
            num_pieces: 0,
            piece_length: 0,
            total_size: 0,
            files: Vec::new(),
            orig_files: Vec::new(),
            pieces: None,
            file_priorities: Vec::new(),
            file_progress: Vec::new(),
            tracker: String::new(),
            tracker_host: String::new(),
            tracker_status: String::new(),
            trackers: Vec::new(),
            auto_managed: false,
            is_auto_managed: false,
            download_location: String::new(),
            max_connections: None,
            max_download_speed: None,
            max_upload_slots: None,
            max_upload_speed: None,
            move_completed: false,
            move_completed_path: String::new(),
            owner: String::new(),
            prioritize_first_last_pieces: false,
            remove_at_ratio: false,
            sequential_download: false,
            shared: false,
            stop_at_ratio: false,
            stop_ratio: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use std::collections::BTreeMap;

    fn make_full_status_dict() -> RencodeValue {
        let mut map = BTreeMap::new();

        // Time / transfer stats
        map.insert(
            RencodeValue::Str("active_time".into()),
            RencodeValue::Int(3600),
        );
        map.insert(
            RencodeValue::Str("all_time_download".into()),
            RencodeValue::Int(1_073_741_824),
        );
        map.insert(
            RencodeValue::Str("completed_time".into()),
            RencodeValue::Int(1_700_000_000),
        );
        map.insert(
            RencodeValue::Str("finished_time".into()),
            RencodeValue::Int(1800),
        );
        map.insert(
            RencodeValue::Str("last_seen_complete".into()),
            RencodeValue::Int(1_700_000_000),
        );
        map.insert(
            RencodeValue::Str("seeding_time".into()),
            RencodeValue::Int(7200),
        );
        map.insert(
            RencodeValue::Str("time_added".into()),
            RencodeValue::Int(1_699_000_000),
        );
        map.insert(
            RencodeValue::Str("time_since_download".into()),
            RencodeValue::Int(60),
        );
        map.insert(
            RencodeValue::Str("time_since_transfer".into()),
            RencodeValue::Int(30),
        );
        map.insert(
            RencodeValue::Str("time_since_upload".into()),
            RencodeValue::Int(120),
        );
        map.insert(
            RencodeValue::Str("total_done".into()),
            RencodeValue::Int(536_870_912),
        );
        map.insert(
            RencodeValue::Str("total_payload_download".into()),
            RencodeValue::Int(536_870_912),
        );
        map.insert(
            RencodeValue::Str("total_payload_upload".into()),
            RencodeValue::Int(1_073_741_824),
        );
        map.insert(
            RencodeValue::Str("total_remaining".into()),
            RencodeValue::Int(536_870_912),
        );
        map.insert(
            RencodeValue::Str("total_uploaded".into()),
            RencodeValue::Int(1_073_741_824),
        );
        map.insert(
            RencodeValue::Str("total_wanted".into()),
            RencodeValue::Int(1_073_741_824),
        );

        // Rates
        map.insert(
            RencodeValue::Str("download_payload_rate".into()),
            RencodeValue::Int(1_048_576),
        );
        map.insert(
            RencodeValue::Str("upload_payload_rate".into()),
            RencodeValue::Int(524_288),
        );
        map.insert(RencodeValue::Str("eta".into()), RencodeValue::Int(600));

        // Ratios
        map.insert(
            RencodeValue::Str("distributed_copies".into()),
            RencodeValue::Float(2.5),
        );
        map.insert(RencodeValue::Str("ratio".into()), RencodeValue::Float(2.0));
        map.insert(RencodeValue::Str("seed_rank".into()), RencodeValue::Int(3));
        map.insert(
            RencodeValue::Str("seeds_peers_ratio".into()),
            RencodeValue::Float(1.5),
        );

        // Peers / seeds
        map.insert(RencodeValue::Str("num_peers".into()), RencodeValue::Int(5));
        map.insert(RencodeValue::Str("num_seeds".into()), RencodeValue::Int(10));
        map.insert(
            RencodeValue::Str("total_peers".into()),
            RencodeValue::Int(20),
        );
        map.insert(
            RencodeValue::Str("total_seeds".into()),
            RencodeValue::Int(50),
        );
        map.insert(
            RencodeValue::Str("peers".into()),
            RencodeValue::List(vec![]),
        );

        // State
        map.insert(
            RencodeValue::Str("state".into()),
            RencodeValue::Str("Seeding".into()),
        );
        map.insert(
            RencodeValue::Str("paused".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("progress".into()),
            RencodeValue::Float(100.0),
        );
        map.insert(
            RencodeValue::Str("is_seed".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("is_finished".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("seed_mode".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("super_seeding".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("message".into()),
            RencodeValue::Str("OK".into()),
        );
        map.insert(RencodeValue::Str("queue".into()), RencodeValue::Int(1));
        map.insert(
            RencodeValue::Str("storage_mode".into()),
            RencodeValue::Str("sparse".into()),
        );

        // Identity / metadata
        map.insert(
            RencodeValue::Str("hash".into()),
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
        );
        map.insert(
            RencodeValue::Str("name".into()),
            RencodeValue::Str("test-torrent".into()),
        );
        map.insert(
            RencodeValue::Str("comment".into()),
            RencodeValue::Str("a test torrent".into()),
        );
        map.insert(
            RencodeValue::Str("creator".into()),
            RencodeValue::Str("test-creator".into()),
        );
        map.insert(
            RencodeValue::Str("private".into()),
            RencodeValue::Bool(false),
        );
        map.insert(RencodeValue::Str("num_files".into()), RencodeValue::Int(1));
        map.insert(
            RencodeValue::Str("num_pieces".into()),
            RencodeValue::Int(1024),
        );
        map.insert(
            RencodeValue::Str("piece_length".into()),
            RencodeValue::Int(1_048_576),
        );
        map.insert(
            RencodeValue::Str("total_size".into()),
            RencodeValue::Int(1_073_741_824),
        );
        map.insert(
            RencodeValue::Str("files".into()),
            RencodeValue::List(vec![]),
        );
        map.insert(
            RencodeValue::Str("orig_files".into()),
            RencodeValue::List(vec![]),
        );
        map.insert(RencodeValue::Str("pieces".into()), RencodeValue::None);

        // File state
        map.insert(
            RencodeValue::Str("file_priorities".into()),
            RencodeValue::List(vec![RencodeValue::Int(2)]),
        );
        map.insert(
            RencodeValue::Str("file_progress".into()),
            RencodeValue::List(vec![RencodeValue::Float(1.0)]),
        );

        // Trackers
        map.insert(
            RencodeValue::Str("tracker".into()),
            RencodeValue::Str("http://tracker.example.com:6969/announce".into()),
        );
        map.insert(
            RencodeValue::Str("tracker_host".into()),
            RencodeValue::Str("tracker.example.com".into()),
        );
        map.insert(
            RencodeValue::Str("tracker_status".into()),
            RencodeValue::Str("Announce OK".into()),
        );
        map.insert(
            RencodeValue::Str("trackers".into()),
            RencodeValue::List(vec![]),
        );

        // Options
        map.insert(
            RencodeValue::Str("auto_managed".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("is_auto_managed".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("download_location".into()),
            RencodeValue::Str("/downloads".into()),
        );
        map.insert(
            RencodeValue::Str("max_connections".into()),
            RencodeValue::Int(200),
        );
        map.insert(
            RencodeValue::Str("max_download_speed".into()),
            RencodeValue::Float(1024.0),
        );
        map.insert(
            RencodeValue::Str("max_upload_slots".into()),
            RencodeValue::Int(4),
        );
        map.insert(
            RencodeValue::Str("max_upload_speed".into()),
            RencodeValue::Float(512.0),
        );
        map.insert(
            RencodeValue::Str("move_completed".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("move_completed_path".into()),
            RencodeValue::Str("/completed".into()),
        );
        map.insert(
            RencodeValue::Str("owner".into()),
            RencodeValue::Str("admin".into()),
        );
        map.insert(
            RencodeValue::Str("prioritize_first_last_pieces".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("remove_at_ratio".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("sequential_download".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("shared".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("stop_at_ratio".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("stop_ratio".into()),
            RencodeValue::Float(2.0),
        );

        RencodeValue::Dict(map)
    }

    #[test]
    fn when_full_status_dict_then_all_fields_populate() {
        let value = make_full_status_dict();

        let result: TorrentStatus = TorrentStatus::deserialize(&value).expect("deserialize");

        assert_eq!(result.active_time, 3600);
        assert_eq!(result.all_time_download, 1_073_741_824);
        assert_eq!(result.completed_time, Some(1_700_000_000));
        assert_eq!(result.finished_time, 1800);
        assert_eq!(result.last_seen_complete, Some(1_700_000_000));
        assert_eq!(result.seeding_time, 7200);
        assert_eq!(result.time_added, 1_699_000_000);
        assert_eq!(result.time_since_download, Some(60));
        assert_eq!(result.time_since_transfer, Some(30));
        assert_eq!(result.time_since_upload, Some(120));
        assert_eq!(result.total_done, 536_870_912);
        assert_eq!(result.total_payload_download, 536_870_912);
        assert_eq!(result.total_payload_upload, 1_073_741_824);
        assert_eq!(result.total_remaining, 536_870_912);
        assert_eq!(result.total_uploaded, 1_073_741_824);
        assert_eq!(result.total_wanted, 1_073_741_824);

        assert_eq!(result.download_payload_rate, 1_048_576);
        assert_eq!(result.upload_payload_rate, 524_288);
        assert_eq!(result.eta, Some(600));

        assert!((result.distributed_copies - 2.5).abs() < f64::EPSILON);
        assert!((result.ratio.unwrap() - 2.0).abs() < f64::EPSILON);
        assert_eq!(result.seed_rank, 3);
        assert!((result.seeds_peers_ratio.unwrap() - 1.5).abs() < f64::EPSILON);

        assert_eq!(result.num_peers, 5);
        assert_eq!(result.num_seeds, 10);
        assert_eq!(result.total_peers, 20);
        assert_eq!(result.total_seeds, 50);
        assert!(result.peers.is_empty());

        assert_eq!(result.state, "Seeding");
        assert!(!result.paused);
        assert!((result.progress - 100.0).abs() < f64::EPSILON);
        assert!(result.is_seed);
        assert!(result.is_finished);
        assert!(!result.seed_mode);
        assert!(!result.super_seeding);
        assert_eq!(result.message, "OK");
        assert_eq!(result.queue, 1);
        assert_eq!(result.storage_mode, "sparse");

        assert_eq!(result.hash, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
        assert_eq!(result.name, "test-torrent");
        assert_eq!(result.comment, "a test torrent");
        assert_eq!(result.creator, "test-creator");
        assert!(!result.private);
        assert_eq!(result.num_files, 1);
        assert_eq!(result.num_pieces, 1024);
        assert_eq!(result.piece_length, 1_048_576);
        assert_eq!(result.total_size, 1_073_741_824);
        assert!(result.files.is_empty());
        assert!(result.orig_files.is_empty());
        assert_eq!(result.pieces, None);

        assert_eq!(result.file_priorities, vec![2]);
        assert!((result.file_progress[0] - 1.0).abs() < f64::EPSILON);

        assert_eq!(result.tracker, "http://tracker.example.com:6969/announce");
        assert_eq!(result.tracker_host, "tracker.example.com");
        assert_eq!(result.tracker_status, "Announce OK");
        assert!(result.trackers.is_empty());

        assert!(result.auto_managed);
        assert!(result.is_auto_managed);
        assert_eq!(result.download_location, "/downloads");
        assert_eq!(result.max_connections, Some(200));
        assert!((result.max_download_speed.unwrap() - 1024.0).abs() < f64::EPSILON);
        assert_eq!(result.max_upload_slots, Some(4));
        assert!((result.max_upload_speed.unwrap() - 512.0).abs() < f64::EPSILON);
        assert!(!result.move_completed);
        assert_eq!(result.move_completed_path, "/completed");
        assert_eq!(result.owner, "admin");
        assert!(result.prioritize_first_last_pieces);
        assert!(!result.remove_at_ratio);
        assert!(!result.sequential_download);
        assert!(!result.shared);
        assert!(!result.stop_at_ratio);
        assert!((result.stop_ratio - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn when_sentinel_minus_one_ratio_then_none() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("ratio".into()), RencodeValue::Float(-1.0));
        let value = RencodeValue::Dict(map);

        let result: TorrentStatus = TorrentStatus::deserialize(&value).expect("deserialize");

        assert_eq!(result.ratio, None);
    }

    #[test]
    fn when_sentinel_minus_one_eta_then_none() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("eta".into()), RencodeValue::Int(-1));
        let value = RencodeValue::Dict(map);

        let result: TorrentStatus = TorrentStatus::deserialize(&value).expect("deserialize");

        assert_eq!(result.eta, None);
    }

    #[test]
    fn when_sentinel_minus_one_completed_time_then_none() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("completed_time".into()),
            RencodeValue::Int(-1),
        );
        let value = RencodeValue::Dict(map);

        let result: TorrentStatus = TorrentStatus::deserialize(&value).expect("deserialize");

        assert_eq!(result.completed_time, None);
    }

    #[test]
    fn when_sentinel_minus_one_max_connections_then_none() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("max_connections".into()),
            RencodeValue::Int(-1),
        );
        let value = RencodeValue::Dict(map);

        let result: TorrentStatus = TorrentStatus::deserialize(&value).expect("deserialize");

        assert_eq!(result.max_connections, None);
    }

    #[test]
    fn when_sentinel_minus_one_max_download_speed_then_none() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("max_download_speed".into()),
            RencodeValue::Float(-1.0),
        );
        let value = RencodeValue::Dict(map);

        let result: TorrentStatus = TorrentStatus::deserialize(&value).expect("deserialize");

        assert_eq!(result.max_download_speed, None);
    }

    #[test]
    fn when_missing_optional_field_then_default() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("name".into()),
            RencodeValue::Str("minimal".into()),
        );
        let value = RencodeValue::Dict(map);

        let result: TorrentStatus = TorrentStatus::deserialize(&value).expect("deserialize");

        assert_eq!(result.name, "minimal");
        assert_eq!(result.comment, "");
        assert_eq!(result.creator, "");
        assert!(!result.private);
        assert_eq!(result.num_files, 0);
        assert!(result.files.is_empty());
        assert!(result.orig_files.is_empty());
        assert_eq!(result.pieces, None);
        assert!(result.file_priorities.is_empty());
        assert!(result.file_progress.is_empty());
        assert_eq!(result.tracker, "");
        assert_eq!(result.tracker_host, "");
        assert_eq!(result.tracker_status, "");
        assert!(result.trackers.is_empty());
        assert_eq!(result.move_completed_path, "");
        assert_eq!(result.owner, "");
        assert_eq!(result.message, "");
    }
}
