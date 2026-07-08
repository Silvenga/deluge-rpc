use super::sub_dicts::{FileInfo, PeerInfo, TrackerInfo};
use crate::sentinels::{
    deserialize_never_i64, deserialize_ratio, deserialize_unlimited_f64, deserialize_unlimited_i64,
};
use serde::{Deserialize, Serialize};

/// Status of a single torrent, returned by `core.get_torrent_status` / `core.get_torrents_status`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct TorrentStatus {
    /// Seconds since the torrent was added.
    pub active_time: i64,
    /// Total bytes downloaded (all-time).
    pub all_time_download: i64,
    /// Unix timestamp when download completed; `None` if not completed.
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub completed_time: Option<i64>,
    /// Seconds the torrent has been in finished state.
    pub finished_time: i64,
    /// Unix timestamp when torrent was last seen complete; `None` if never.
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub last_seen_complete: Option<i64>,
    /// Seconds spent seeding.
    pub seeding_time: i64,
    /// Unix timestamp when the torrent was added.
    pub time_added: i64,
    /// Seconds since last download activity; `None` if never.
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub time_since_download: Option<i64>,
    /// Seconds since last upload or download; `None` if never.
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub time_since_transfer: Option<i64>,
    /// Seconds since last upload activity; `None` if never.
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub time_since_upload: Option<i64>,
    /// Bytes downloaded and verified (pieces).
    pub total_done: i64,
    /// Payload bytes downloaded (excludes protocol overhead).
    pub total_payload_download: i64,
    /// Payload bytes uploaded (excludes protocol overhead).
    pub total_payload_upload: i64,
    /// Bytes remaining to download.
    pub total_remaining: i64,
    /// Total bytes uploaded (all-time).
    pub total_uploaded: i64,
    /// Bytes of files marked for download (excludes skipped files).
    pub total_wanted: i64,

    /// Current payload download speed in bytes/sec.
    pub download_payload_rate: i64,
    /// Current payload upload speed in bytes/sec.
    pub upload_payload_rate: i64,
    /// Estimated seconds to completion; `None` if >1 year, `0` if idle.
    #[serde(deserialize_with = "deserialize_never_i64", default)]
    pub eta: Option<i64>,

    /// Distributed copies (swarm availability); `>= 0.0`.
    pub distributed_copies: f64,
    /// Share ratio; `None` means infinity (when `total_done == 0`).
    #[serde(deserialize_with = "deserialize_ratio", default)]
    pub ratio: Option<f64>,
    /// Seed rank score used for queue ordering.
    pub seed_rank: i64,
    /// Seeds-to-peers ratio; `None` if no incomplete peers.
    #[serde(deserialize_with = "deserialize_ratio", default)]
    pub seeds_peers_ratio: Option<f64>,

    /// Connected peers (excludes seeds).
    pub num_peers: i64,
    /// Connected seeds.
    pub num_seeds: i64,
    /// Total peers in swarm (unconnected, from tracker).
    pub total_peers: i64,
    /// Total seeds in swarm (unconnected, from tracker).
    pub total_seeds: i64,
    /// Connected peers.
    #[serde(default)]
    pub peers: Vec<PeerInfo>,

    /// Current state. One of: `Allocating`, `Checking`, `Downloading`, `Seeding`, `Paused`, `Error`, `Queued`, `Moving`.
    pub state: String,
    /// Whether the torrent is paused.
    pub paused: bool,
    /// Download progress as `0.0`–`100.0`.
    pub progress: f64,
    /// Whether the torrent is seeding (download complete).
    pub is_seed: bool,
    /// Whether the torrent has finished downloading.
    pub is_finished: bool,
    /// Whether the torrent is in seed mode (started assuming all data present).
    pub seed_mode: bool,
    /// Whether super seeding is enabled.
    pub super_seeding: bool,
    /// Status message; default `'OK'`.
    #[serde(default)]
    pub message: String,
    /// Queue position.
    pub queue: i64,
    /// Storage allocation mode: `'sparse'` or `'allocate'`.
    pub storage_mode: String,

    /// Info hash (same as torrent_id).
    pub hash: String,
    /// Display name.
    pub name: String,
    /// Torrent comment; `''` if no metadata.
    #[serde(default)]
    pub comment: String,
    /// Torrent creator; `''` if no metadata.
    #[serde(default)]
    pub creator: String,
    /// Whether the torrent is private; `False` if no metadata.
    #[serde(default)]
    pub private: bool,
    /// Number of files; `0` if no metadata.
    #[serde(default)]
    pub num_files: i64,
    /// Number of pieces; `0` if no metadata.
    #[serde(default)]
    pub num_pieces: i64,
    /// Piece length in bytes; `0` if no metadata.
    #[serde(default)]
    pub piece_length: i64,
    /// Total size of all files in bytes; `0` if no metadata.
    #[serde(default)]
    pub total_size: i64,
    /// File list. `[]` if no metadata.
    #[serde(default)]
    pub files: Vec<FileInfo>,
    /// Original file paths (before rename). `[]` if no metadata.
    #[serde(default)]
    pub orig_files: Vec<FileInfo>,
    /// Per-piece state. `None` if seeding or no metadata. Else: `0`=missing, `1`=available, `2`=downloading, `3`=completed.
    #[serde(default)]
    pub pieces: Option<Vec<i64>>,

    /// Per-file priority: `0`=skip, `1`=low, `2`=normal, `5`=high, `7`=highest. `[]` if no metadata.
    #[serde(default)]
    pub file_priorities: Vec<i64>,
    /// Per-file progress `0.0`–`1.0`. `[]` if no metadata.
    #[serde(default)]
    pub file_progress: Vec<f64>,

    /// Current tracker URL.
    #[serde(default)]
    pub tracker: String,
    /// Short hostname of current tracker.
    #[serde(default)]
    pub tracker_host: String,
    /// Tracker status string, e.g. `'Announce OK'` or `'Error: ...'`.
    #[serde(default)]
    pub tracker_status: String,
    /// Full tracker list.
    #[serde(default)]
    pub trackers: Vec<TrackerInfo>,

    /// Whether the torrent is auto-managed by the queue.
    pub auto_managed: bool,
    /// Alias for `auto_managed`.
    pub is_auto_managed: bool,
    /// Save path.
    pub download_location: String,
    /// Max connections; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_i64", default)]
    pub max_connections: Option<i64>,
    /// Max download speed in KiB/s; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_f64", default)]
    pub max_download_speed: Option<f64>,
    /// Max upload slots; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_i64", default)]
    pub max_upload_slots: Option<i64>,
    /// Max upload speed in KiB/s; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_f64", default)]
    pub max_upload_speed: Option<f64>,
    /// Whether to move when completed.
    pub move_completed: bool,
    /// Destination path for move-on-completion.
    #[serde(default)]
    pub move_completed_path: String,
    /// Username of the torrent owner.
    #[serde(default)]
    pub owner: String,
    /// Whether to prioritize first and last pieces.
    pub prioritize_first_last_pieces: bool,
    /// Whether to remove the torrent when stop ratio is reached.
    pub remove_at_ratio: bool,
    /// Whether to download pieces sequentially.
    pub sequential_download: bool,
    /// Whether the torrent is shared across users.
    pub shared: bool,
    /// Whether to stop at the stop ratio.
    pub stop_at_ratio: bool,
    /// The stop ratio.
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
    use deluge_rpc_rencode::RencodeValue;
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
