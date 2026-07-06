use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Filter dict for `core.get_torrents_status`.
///
/// All filter values are lists even for a single value. `{}` = no filter (return all).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FilterDict {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracker_host: Option<Vec<String>>,
}

/// Return of `core.get_filter_tree`: `{field: [(value, count), ...]}`.
pub type FilterTree = BTreeMap<String, Vec<FilterTreeEntry>>;

/// A single entry in the filter tree: `(value: str, count: int)`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FilterTreeEntry {
    pub value: String,
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_filter_dict_default_then_all_fields_none() {
        let filter = FilterDict::default();

        assert_eq!(filter.id, None);
        assert_eq!(filter.state, None);
        assert_eq!(filter.keyword, None);
        assert_eq!(filter.name, None);
        assert_eq!(filter.tracker_host, None);
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
}
