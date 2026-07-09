use crate::deserialize_unlimited_i64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the AutoAdd plugin.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AutoAddConfig {
    /// All watch folders keyed by ID.
    #[serde(rename = "watchdirs")]
    pub watch_dirs: HashMap<String, WatchDirOptions>,
    /// Next ID to assign to a new watchdir.
    pub next_id: i64,
}

/// Numeric identifier for a watch directory.
pub type WatchDirId = i64;

/// Options for a single AutoAdd watch directory.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct WatchDirOptions {
    /// Whether the watchdir is actively polling.
    pub enabled: bool,
    /// Filesystem path to watch for .torrent files.
    pub path: String,
    /// Extension to append to processed .torrent files.
    pub append_extension: String,
    /// Whether to copy .torrent files to torrentfiles_location.
    pub copy_torrent: bool,
    /// Whether to delete copied .torrent files after processing.
    pub delete_copy_torrent_toggle: bool,
    /// Whether to use absolute paths.
    #[serde(rename = "abspath")]
    pub abs_path: bool,
    /// Save path for torrents added from this watchdir.
    pub download_location: String,
    /// Per-torrent max download speed; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_download_speed: Option<i64>,
    /// Per-torrent max upload speed; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_upload_speed: Option<i64>,
    /// Per-torrent max connections; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_connections: Option<i64>,
    /// Per-torrent max upload slots; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_upload_slots: Option<i64>,
    /// Whether to prioritize first/last pieces.
    pub prioritize_first_last: bool,
    /// Whether torrents are auto-managed.
    pub auto_managed: bool,
    /// Whether to stop at stop ratio.
    pub stop_at_ratio: bool,
    /// The stop ratio.
    pub stop_ratio: f64,
    /// Whether to remove at stop ratio.
    pub remove_at_ratio: bool,
    /// Whether to move on completion.
    pub move_completed: bool,
    /// Move-on-completion destination.
    pub move_completed_path: String,
    /// Label to apply (requires Label plugin).
    pub label: String,
    /// Whether to add torrents paused.
    pub add_paused: bool,
    /// Whether to place new torrents at queue top.
    pub queue_to_top: bool,
    /// Username of the watchdir owner.
    pub owner: String,
    /// Whether to add torrents in seed mode.
    pub seed_mode: bool,
    /// Toggle: whether max_download_speed is applied when adding.
    pub max_download_speed_toggle: bool,
    /// Toggle: whether max_upload_speed is applied when adding.
    pub max_upload_speed_toggle: bool,
    /// Toggle: whether max_connections is applied when adding.
    pub max_connections_toggle: bool,
    /// Toggle: whether max_upload_slots is applied when adding.
    pub max_upload_slots_toggle: bool,
    /// Toggle: whether download_location is applied when adding.
    pub download_location_toggle: bool,
    /// Toggle: whether move_completed is applied when adding.
    pub move_completed_toggle: bool,
    /// Toggle: whether move_completed_path is applied when adding.
    pub move_completed_path_toggle: bool,
    /// Toggle: whether add_paused is applied when adding.
    pub add_paused_toggle: bool,
    /// Toggle: whether queue_to_top is applied when adding.
    pub queue_to_top_toggle: bool,
    /// Toggle: whether auto_managed is applied when adding.
    pub auto_managed_toggle: bool,
    /// Toggle: whether stop_at_ratio is applied when adding.
    pub stop_at_ratio_toggle: bool,
    /// Toggle: whether stop_ratio is applied when adding.
    pub stop_ratio_toggle: bool,
    /// Toggle: whether remove_at_ratio is applied when adding.
    pub remove_at_ratio_toggle: bool,
    /// Toggle: whether prioritize_first_last is applied when adding.
    pub prioritize_first_last_toggle: bool,
    /// Toggle: whether seed_mode is applied when adding.
    pub seed_mode_toggle: bool,
    /// Toggle: whether label is applied when adding.
    pub label_toggle: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rpc_rencode::RencodeValue;
    use std::collections::BTreeMap;

    fn make_dict(entries: Vec<(&str, RencodeValue)>) -> RencodeValue {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(RencodeValue::Str(k.into()), v);
        }
        RencodeValue::Dict(map)
    }

    fn make_watchdir_options_dict() -> RencodeValue {
        make_dict(vec![
            ("enabled", RencodeValue::Bool(true)),
            ("path", RencodeValue::Str("/watch/torrents".into())),
            ("append_extension", RencodeValue::Str(".added".into())),
            ("copy_torrent", RencodeValue::Bool(false)),
            ("delete_copy_torrent_toggle", RencodeValue::Bool(false)),
            ("abspath", RencodeValue::Bool(true)),
            ("download_location", RencodeValue::Str("/downloads".into())),
            ("max_download_speed", RencodeValue::Int(-1)),
            ("max_upload_speed", RencodeValue::Int(-1)),
            ("max_connections", RencodeValue::Int(-1)),
            ("max_upload_slots", RencodeValue::Int(-1)),
            ("prioritize_first_last", RencodeValue::Bool(false)),
            ("auto_managed", RencodeValue::Bool(true)),
            ("stop_at_ratio", RencodeValue::Bool(false)),
            ("stop_ratio", RencodeValue::Float(2.0)),
            ("remove_at_ratio", RencodeValue::Bool(false)),
            ("move_completed", RencodeValue::Bool(false)),
            ("move_completed_path", RencodeValue::Str("".into())),
            ("label", RencodeValue::Str("".into())),
            ("add_paused", RencodeValue::Bool(false)),
            ("queue_to_top", RencodeValue::Bool(false)),
            ("owner", RencodeValue::Str("admin".into())),
            ("seed_mode", RencodeValue::Bool(false)),
            ("max_download_speed_toggle", RencodeValue::Bool(false)),
            ("max_upload_speed_toggle", RencodeValue::Bool(false)),
            ("max_connections_toggle", RencodeValue::Bool(false)),
            ("max_upload_slots_toggle", RencodeValue::Bool(false)),
            ("download_location_toggle", RencodeValue::Bool(false)),
            ("move_completed_toggle", RencodeValue::Bool(false)),
            ("move_completed_path_toggle", RencodeValue::Bool(false)),
            ("add_paused_toggle", RencodeValue::Bool(false)),
            ("queue_to_top_toggle", RencodeValue::Bool(false)),
            ("auto_managed_toggle", RencodeValue::Bool(false)),
            ("stop_at_ratio_toggle", RencodeValue::Bool(false)),
            ("stop_ratio_toggle", RencodeValue::Bool(false)),
            ("remove_at_ratio_toggle", RencodeValue::Bool(false)),
            ("prioritize_first_last_toggle", RencodeValue::Bool(false)),
            ("seed_mode_toggle", RencodeValue::Bool(false)),
            ("label_toggle", RencodeValue::Bool(false)),
        ])
    }

    #[test]
    fn when_autoadd_config_dict_then_fields_populate() {
        let mut watchdirs = BTreeMap::new();
        watchdirs.insert(RencodeValue::Str("1".into()), make_watchdir_options_dict());

        let value = make_dict(vec![
            ("watchdirs", RencodeValue::Dict(watchdirs)),
            ("next_id", RencodeValue::Int(2)),
        ]);

        let result: AutoAddConfig = AutoAddConfig::deserialize(&value).expect("deserialize");

        assert_eq!(result.next_id, 2);
        assert_eq!(result.watch_dirs.len(), 1);
        let opts = result.watch_dirs.get("1").expect("watchdir 1");
        assert!(opts.enabled);
        assert_eq!(opts.path, "/watch/torrents");
        assert_eq!(opts.owner, "admin");
    }

    #[test]
    fn when_watchdir_options_dict_then_fields_populate() {
        let value = make_watchdir_options_dict();

        let result: WatchDirOptions = WatchDirOptions::deserialize(&value).expect("deserialize");

        assert!(result.enabled);
        assert_eq!(result.path, "/watch/torrents");
        assert_eq!(result.append_extension, ".added");
        assert!(!result.copy_torrent);
        assert!(!result.delete_copy_torrent_toggle);
        assert!(result.abs_path);
        assert_eq!(result.download_location, "/downloads");
        assert_eq!(result.max_download_speed, None);
        assert_eq!(result.max_upload_speed, None);
        assert_eq!(result.max_connections, None);
        assert_eq!(result.max_upload_slots, None);
        assert!(!result.prioritize_first_last);
        assert!(result.auto_managed);
        assert!(!result.stop_at_ratio);
        assert!((result.stop_ratio - 2.0).abs() < f64::EPSILON);
        assert!(!result.remove_at_ratio);
        assert!(!result.move_completed);
        assert_eq!(result.move_completed_path, "");
        assert_eq!(result.label, "");
        assert!(!result.add_paused);
        assert!(!result.queue_to_top);
        assert_eq!(result.owner, "admin");
        assert!(!result.seed_mode);
    }

    #[test]
    fn when_watchdir_options_speed_limited_then_some() {
        let value = make_dict(vec![
            ("enabled", RencodeValue::Bool(true)),
            ("path", RencodeValue::Str("/watch".into())),
            ("append_extension", RencodeValue::Str("".into())),
            ("copy_torrent", RencodeValue::Bool(false)),
            ("delete_copy_torrent_toggle", RencodeValue::Bool(false)),
            ("abspath", RencodeValue::Bool(false)),
            ("download_location", RencodeValue::Str("".into())),
            ("max_download_speed", RencodeValue::Int(1000)),
            ("max_upload_speed", RencodeValue::Int(500)),
            ("max_connections", RencodeValue::Int(200)),
            ("max_upload_slots", RencodeValue::Int(10)),
            ("prioritize_first_last", RencodeValue::Bool(false)),
            ("auto_managed", RencodeValue::Bool(true)),
            ("stop_at_ratio", RencodeValue::Bool(false)),
            ("stop_ratio", RencodeValue::Float(2.0)),
            ("remove_at_ratio", RencodeValue::Bool(false)),
            ("move_completed", RencodeValue::Bool(false)),
            ("move_completed_path", RencodeValue::Str("".into())),
            ("label", RencodeValue::Str("".into())),
            ("add_paused", RencodeValue::Bool(false)),
            ("queue_to_top", RencodeValue::Bool(false)),
            ("owner", RencodeValue::Str("admin".into())),
            ("seed_mode", RencodeValue::Bool(false)),
            ("max_download_speed_toggle", RencodeValue::Bool(false)),
            ("max_upload_speed_toggle", RencodeValue::Bool(false)),
            ("max_connections_toggle", RencodeValue::Bool(false)),
            ("max_upload_slots_toggle", RencodeValue::Bool(false)),
            ("download_location_toggle", RencodeValue::Bool(false)),
            ("move_completed_toggle", RencodeValue::Bool(false)),
            ("move_completed_path_toggle", RencodeValue::Bool(false)),
            ("add_paused_toggle", RencodeValue::Bool(false)),
            ("queue_to_top_toggle", RencodeValue::Bool(false)),
            ("auto_managed_toggle", RencodeValue::Bool(false)),
            ("stop_at_ratio_toggle", RencodeValue::Bool(false)),
            ("stop_ratio_toggle", RencodeValue::Bool(false)),
            ("remove_at_ratio_toggle", RencodeValue::Bool(false)),
            ("prioritize_first_last_toggle", RencodeValue::Bool(false)),
            ("seed_mode_toggle", RencodeValue::Bool(false)),
            ("label_toggle", RencodeValue::Bool(false)),
        ]);

        let result: WatchDirOptions = WatchDirOptions::deserialize(&value).expect("deserialize");

        assert_eq!(result.max_download_speed, Some(1000));
        assert_eq!(result.max_upload_speed, Some(500));
        assert_eq!(result.max_connections, Some(200));
        assert_eq!(result.max_upload_slots, Some(10));
    }
}
