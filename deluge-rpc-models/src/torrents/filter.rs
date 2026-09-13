use crate::raw::RawList;
use deluge_rpc_rencode::RencodeValue;
use serde::ser::{Error as _, SerializeMap};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeMap;

/// Filter dict for `core.get_torrents_status`.
///
/// All filter values are lists even for a single value. `{}` = no filter (return all).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FilterDict {
    /// Direct torrent_id match (optimized path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<String>>,
    /// State values. `"Active"` filters by `download_payload_rate > 0 or upload_payload_rate > 0`; other values match `torrent.state` directly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<Vec<String>>,
    /// Searches name, state, tracker URL, info_hash, tracker_status, and file paths (case-insensitive, comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<Vec<String>>,
    /// Substring match on torrent name. `"::match"` suffix = case-sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Vec<String>>,
    /// Matches `tracker_host` status field. `"Error"` matches torrents with `"Error:"` in `tracker_status`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracker_host: Option<Vec<String>>,

    /// Additional filters with arbitrary list values, e.g. plugin-provided fields.
    /// Keys colliding with a named field are rejected on serialization.
    #[serde(flatten, serialize_with = "serialize_extra")]
    pub extra: BTreeMap<String, Vec<RencodeValue>>,
}

/// Keys owned by the named fields. `extra` entries using these keys are rejected on serialization
/// to avoid emitting duplicate keys.
const RESERVED_KEYS: &[&str] = &["id", "state", "keyword", "name", "tracker_host"];

fn serialize_extra<S: Serializer>(
    extra: &BTreeMap<String, Vec<RencodeValue>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut map = serializer.serialize_map(Some(extra.len()))?;
    for (key, values) in extra {
        if RESERVED_KEYS.contains(&key.as_str()) {
            return Err(S::Error::custom(format!(
                "extra key `{key}` collides with a named FilterDict field"
            )));
        }
        map.serialize_entry(key, &RawList(values))?;
    }
    map.end()
}

/// Return of `core.get_filter_tree`: `{field: [(value, count), ...]}`.
pub type FilterTree = BTreeMap<String, Vec<FilterTreeEntry>>;

/// A single entry in the filter tree: `(value: str, count: int)`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FilterTreeEntry {
    /// Filter value string.
    pub value: String,
    /// Number of torrents matching this value.
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rpc_rencode::to_rencode_value;

    #[test]
    fn when_filter_dict_default_then_all_fields_none() {
        let filter = FilterDict::default();

        assert_eq!(filter.id, None);
        assert_eq!(filter.state, None);
        assert_eq!(filter.keyword, None);
        assert_eq!(filter.name, None);
        assert_eq!(filter.tracker_host, None);
        assert!(filter.extra.is_empty());
    }

    #[test]
    fn when_filter_dict_typed_then_correct_args() {
        let filter = FilterDict {
            state: Some(vec!["Seeding".into(), "Downloading".into()]),
            ..Default::default()
        };

        let json = serde_json::to_value(&filter).expect("serialize");

        assert_eq!(json["state"][0], "Seeding");
        assert_eq!(json["state"][1], "Downloading");
        assert!(json.get("id").is_none());
    }

    #[test]
    fn when_filter_dict_deserialize_then_all_fields() {
        let json = serde_json::json!({
            "id": ["aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111"],
            "state": ["Active"],
            "keyword": ["ubuntu"],
            "name": ["test"],
            "tracker_host": ["tracker.example.com"]
        });

        let filter: FilterDict = serde_json::from_value(json).expect("deserialize");

        assert_eq!(
            filter.id,
            Some(vec!["aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()])
        );
        assert_eq!(filter.state, Some(vec!["Active".into()]));
        assert_eq!(filter.keyword, Some(vec!["ubuntu".into()]));
        assert_eq!(filter.name, Some(vec!["test".into()]));
        assert_eq!(
            filter.tracker_host,
            Some(vec!["tracker.example.com".into()])
        );
    }

    #[test]
    fn when_filter_tree_entry_deserialize_then_fields_populate() {
        let json = serde_json::json!(["All", 42]);

        let entry: FilterTreeEntry = serde_json::from_value(json).expect("deserialize");

        assert_eq!(entry.value, "All");
        assert_eq!(entry.count, 42);
    }

    #[test]
    fn when_extra_has_non_string_values_then_rencode_serializes_them() {
        let mut filter = FilterDict::default();
        filter.extra.insert(
            "plugin.ids".into(),
            vec![RencodeValue::Int(1), RencodeValue::Int(2)],
        );

        let value = to_rencode_value(&filter).expect("serialize");

        let RencodeValue::Dict(map) = value else {
            panic!("expected dict");
        };
        assert_eq!(
            map.get(&RencodeValue::Str("plugin.ids".into())),
            Some(&RencodeValue::List(vec![
                RencodeValue::Int(1),
                RencodeValue::Int(2)
            ]))
        );
    }

    #[test]
    fn when_extra_has_non_string_values_then_json_serializes_them() {
        let mut filter = FilterDict::default();
        filter.extra.insert(
            "plugin.ids".into(),
            vec![RencodeValue::Int(1), RencodeValue::Int(2)],
        );

        let json = serde_json::to_value(&filter).expect("serialize");

        assert_eq!(json["plugin.ids"], serde_json::json!([1, 2]));
    }

    #[test]
    fn when_extra_deserialized_then_non_string_values_preserved() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("plugin.ids".into()),
            RencodeValue::List(vec![RencodeValue::Int(1), RencodeValue::Int(2)]),
        );
        let value = RencodeValue::Dict(map);

        let filter: FilterDict = FilterDict::deserialize(&value).expect("deserialize");

        assert_eq!(
            filter.extra.get("plugin.ids"),
            Some(&vec![RencodeValue::Int(1), RencodeValue::Int(2)])
        );
    }

    #[test]
    fn when_extra_key_collides_with_named_field_then_rencode_serialization_rejected() {
        let mut filter = FilterDict::default();
        filter
            .extra
            .insert("state".into(), vec![RencodeValue::Str("Seeding".into())]);

        let result = to_rencode_value(&filter);

        assert!(result.is_err());
    }

    #[test]
    fn when_extra_key_collides_with_named_field_then_json_serialization_rejected() {
        let mut filter = FilterDict::default();
        filter
            .extra
            .insert("state".into(), vec![RencodeValue::Str("Seeding".into())]);

        let result = serde_json::to_value(&filter);

        assert!(result.is_err());
    }
}
