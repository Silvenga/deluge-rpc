use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AddTorrentOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_priorities: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_download_speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_upload_slots: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_upload_speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_completed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_completed_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prioritize_first_last_pieces: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequential_download: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_managed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_at_ratio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_at_ratio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub super_seeding: Option<bool>,
}

/// Options for `core.set_torrent_options`. Same keys as `AddTorrentOptions`.
pub type SetTorrentOptions = AddTorrentOptions;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_add_torrent_options_default_then_all_fields_none() {
        let opts = AddTorrentOptions::default();

        assert_eq!(opts.file_priorities, None);
        assert_eq!(opts.max_connections, None);
        assert_eq!(opts.max_download_speed, None);
        assert_eq!(opts.max_upload_slots, None);
        assert_eq!(opts.max_upload_speed, None);
        assert_eq!(opts.move_completed, None);
        assert_eq!(opts.move_completed_path, None);
        assert_eq!(opts.name, None);
        assert_eq!(opts.prioritize_first_last_pieces, None);
        assert_eq!(opts.sequential_download, None);
        assert_eq!(opts.download_location, None);
        assert_eq!(opts.add_paused, None);
        assert_eq!(opts.auto_managed, None);
        assert_eq!(opts.owner, None);
        assert_eq!(opts.shared, None);
        assert_eq!(opts.stop_at_ratio, None);
        assert_eq!(opts.stop_ratio, None);
        assert_eq!(opts.remove_at_ratio, None);
        assert_eq!(opts.super_seeding, None);
    }

    #[test]
    fn when_add_torrent_options_all_fields_set_then_serialize_skips_none() {
        let opts = AddTorrentOptions {
            download_location: Some("/downloads".into()),
            add_paused: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_value(&opts).expect("serialize");

        assert_eq!(json["download_location"], "/downloads");
        assert_eq!(json["add_paused"], true);
        assert!(json.get("name").is_none());
        assert!(json.get("max_connections").is_none());
    }

    #[test]
    fn when_add_torrent_options_deserialize_then_all_fields() {
        let json = serde_json::json!({
            "download_location": "/downloads",
            "add_paused": true,
            "auto_managed": false,
            "max_connections": 200,
            "max_download_speed": 1024.0,
            "max_upload_slots": 4,
            "max_upload_speed": 512.0,
            "move_completed": true,
            "move_completed_path": "/completed",
            "name": "renamed",
            "prioritize_first_last_pieces": true,
            "sequential_download": false,
            "owner": "admin",
            "shared": false,
            "stop_at_ratio": true,
            "stop_ratio": 2.0,
            "remove_at_ratio": false,
            "super_seeding": false,
            "file_priorities": [2, 2, 2]
        });

        let opts: AddTorrentOptions = serde_json::from_value(json).expect("deserialize");

        assert_eq!(opts.download_location, Some("/downloads".into()));
        assert_eq!(opts.add_paused, Some(true));
        assert_eq!(opts.auto_managed, Some(false));
        assert_eq!(opts.max_connections, Some(200));
        assert!((opts.max_download_speed.unwrap() - 1024.0).abs() < f64::EPSILON);
        assert_eq!(opts.max_upload_slots, Some(4));
        assert!((opts.max_upload_speed.unwrap() - 512.0).abs() < f64::EPSILON);
        assert_eq!(opts.move_completed, Some(true));
        assert_eq!(opts.move_completed_path, Some("/completed".into()));
        assert_eq!(opts.name, Some("renamed".into()));
        assert_eq!(opts.prioritize_first_last_pieces, Some(true));
        assert_eq!(opts.sequential_download, Some(false));
        assert_eq!(opts.owner, Some("admin".into()));
        assert_eq!(opts.shared, Some(false));
        assert_eq!(opts.stop_at_ratio, Some(true));
        assert!((opts.stop_ratio.unwrap() - 2.0).abs() < f64::EPSILON);
        assert_eq!(opts.remove_at_ratio, Some(false));
        assert_eq!(opts.super_seeding, Some(false));
        assert_eq!(opts.file_priorities, Some(vec![2, 2, 2]));
    }
}
