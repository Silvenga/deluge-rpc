use crate::deserialize_unlimited_i64;
use serde::{Deserialize, Serialize};

/// Per-label options applied to torrents with that label.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LabelOptions {
    /// Whether to apply bandwidth limits.
    pub apply_max: bool,
    /// Max download speed; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_download_speed: Option<i64>,
    /// Max upload speed; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_upload_speed: Option<i64>,
    /// Max connections; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_connections: Option<i64>,
    /// Max upload slots; `None` = unlimited.
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_upload_slots: Option<i64>,
    /// Whether to prioritize first/last pieces.
    pub prioritize_first_last: bool,
    /// Whether to apply queue settings.
    pub apply_queue: bool,
    /// Whether torrents with this label are auto-managed.
    pub is_auto_managed: bool,
    /// Whether to stop at stop ratio.
    pub stop_at_ratio: bool,
    /// The stop ratio.
    pub stop_ratio: f64,
    /// Whether to remove at stop ratio.
    pub remove_at_ratio: bool,
    /// Whether to apply move-on-completion settings.
    pub apply_move_completed: bool,
    /// Whether to move on completion.
    pub move_completed: bool,
    /// Move-on-completion destination.
    pub move_completed_path: String,
    /// Whether to auto-add torrents matching trackers.
    pub auto_add: bool,
    /// Tracker URLs that trigger auto-adding.
    pub auto_add_trackers: Vec<String>,
}

/// Global configuration for the Label plugin.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LabelConfig {
    /// Tracker URLs that trigger auto-adding.
    pub auto_add_trackers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rencode::RencodeValue;
    use std::collections::BTreeMap;

    fn make_dict(entries: Vec<(&str, RencodeValue)>) -> RencodeValue {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(RencodeValue::Str(k.into()), v);
        }
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_label_options_dict_then_fields_populate() {
        let value = make_dict(vec![
            ("apply_max", RencodeValue::Bool(false)),
            ("max_download_speed", RencodeValue::Int(-1)),
            ("max_upload_speed", RencodeValue::Int(-1)),
            ("max_connections", RencodeValue::Int(-1)),
            ("max_upload_slots", RencodeValue::Int(-1)),
            ("prioritize_first_last", RencodeValue::Bool(false)),
            ("apply_queue", RencodeValue::Bool(false)),
            ("is_auto_managed", RencodeValue::Bool(false)),
            ("stop_at_ratio", RencodeValue::Bool(false)),
            ("stop_ratio", RencodeValue::Float(2.0)),
            ("remove_at_ratio", RencodeValue::Bool(false)),
            ("apply_move_completed", RencodeValue::Bool(false)),
            ("move_completed", RencodeValue::Bool(false)),
            ("move_completed_path", RencodeValue::Str("".into())),
            ("auto_add", RencodeValue::Bool(false)),
            ("auto_add_trackers", RencodeValue::List(vec![])),
        ]);

        let result: LabelOptions = LabelOptions::deserialize(&value).expect("deserialize");

        assert!(!result.apply_max);
        assert_eq!(result.max_download_speed, None);
        assert_eq!(result.max_upload_speed, None);
        assert_eq!(result.max_connections, None);
        assert_eq!(result.max_upload_slots, None);
        assert!(!result.prioritize_first_last);
        assert!(!result.apply_queue);
        assert!(!result.is_auto_managed);
        assert!(!result.stop_at_ratio);
        assert!((result.stop_ratio - 2.0).abs() < f64::EPSILON);
        assert!(!result.remove_at_ratio);
        assert!(!result.apply_move_completed);
        assert!(!result.move_completed);
        assert_eq!(result.move_completed_path, "");
        assert!(!result.auto_add);
        assert!(result.auto_add_trackers.is_empty());
    }

    #[test]
    fn when_label_options_speed_limited_then_some() {
        let value = make_dict(vec![
            ("apply_max", RencodeValue::Bool(true)),
            ("max_download_speed", RencodeValue::Int(5000)),
            ("max_upload_speed", RencodeValue::Int(2000)),
            ("max_connections", RencodeValue::Int(300)),
            ("max_upload_slots", RencodeValue::Int(20)),
            ("prioritize_first_last", RencodeValue::Bool(true)),
            ("apply_queue", RencodeValue::Bool(false)),
            ("is_auto_managed", RencodeValue::Bool(false)),
            ("stop_at_ratio", RencodeValue::Bool(false)),
            ("stop_ratio", RencodeValue::Float(2.0)),
            ("remove_at_ratio", RencodeValue::Bool(false)),
            ("apply_move_completed", RencodeValue::Bool(false)),
            ("move_completed", RencodeValue::Bool(false)),
            ("move_completed_path", RencodeValue::Str("".into())),
            ("auto_add", RencodeValue::Bool(false)),
            ("auto_add_trackers", RencodeValue::List(vec![])),
        ]);

        let result: LabelOptions = LabelOptions::deserialize(&value).expect("deserialize");

        assert!(result.apply_max);
        assert_eq!(result.max_download_speed, Some(5000));
        assert_eq!(result.max_upload_speed, Some(2000));
        assert_eq!(result.max_connections, Some(300));
        assert_eq!(result.max_upload_slots, Some(20));
        assert!(result.prioritize_first_last);
    }

    #[test]
    fn when_label_config_dict_then_fields_populate() {
        let value = make_dict(vec![(
            "auto_add_trackers",
            RencodeValue::List(vec![RencodeValue::Str(
                "http://tracker.example.com/announce".into(),
            )]),
        )]);

        let result: LabelConfig = LabelConfig::deserialize(&value).expect("deserialize");

        assert_eq!(
            result.auto_add_trackers,
            vec!["http://tracker.example.com/announce"]
        );
    }
}
