use serde::Deserialize;

/// Configuration for the Blocklist plugin.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BlocklistConfig {
    /// Blocklist download URL.
    pub url: String,
    /// Whether to load the blocklist on daemon startup.
    pub load_on_start: bool,
    /// Days between automatic re-import checks.
    pub check_after_days: i64,
    /// Compression type (e.g. "gzip").
    pub list_compression: String,
    /// Blocklist format (e.g. "text", "p2p", "dat").
    pub list_type: String,
    /// Unix timestamp of last successful import.
    pub last_update: f64,
    /// Number of entries in the current blocklist.
    pub list_size: i64,
    /// Download timeout in seconds.
    pub timeout: i64,
    /// Number of retry attempts on download failure.
    pub try_times: i64,
    /// Whitelisted IPs that bypass the blocklist.
    pub whitelisted: Vec<String>,
}

/// Current import status of the Blocklist plugin.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BlocklistStatus {
    /// Current state: "Downloading", "Importing", or "Idle".
    pub state: String,
    /// Whether the blocklist is current.
    pub up_to_date: bool,
    /// Number of whitelisted IPs.
    pub num_whited: i64,
    /// Number of blocked IPs/ranges.
    pub num_blocked: i64,
    /// Download progress 0.0–1.0.
    pub file_progress: f64,
    /// URL of the current/last download.
    pub file_url: String,
    /// Size of the downloaded file in bytes.
    pub file_size: i64,
    /// Unix timestamp of the downloaded file.
    pub file_date: f64,
    /// Blocklist type (e.g. "p2p", "p2p (gz)").
    pub file_type: String,
    /// Whitelisted IPs.
    pub whitelisted: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use std::collections::BTreeMap;

    fn make_dict(entries: Vec<(&str, RencodeValue)>) -> RencodeValue {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(RencodeValue::Str(k.into()), v);
        }
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_blocklist_config_dict_then_fields_populate() {
        let value = make_dict(vec![
            (
                "url",
                RencodeValue::Str("https://example.com/blocklist.txt".into()),
            ),
            ("load_on_start", RencodeValue::Bool(true)),
            ("check_after_days", RencodeValue::Int(4)),
            ("list_compression", RencodeValue::Str("gzip".into())),
            ("list_type", RencodeValue::Str("text".into())),
            ("last_update", RencodeValue::Float(1_700_000_000.0)),
            ("list_size", RencodeValue::Int(5000)),
            ("timeout", RencodeValue::Int(30)),
            ("try_times", RencodeValue::Int(3)),
            (
                "whitelisted",
                RencodeValue::List(vec![RencodeValue::Str("1.2.3.4".into())]),
            ),
        ]);

        let result: BlocklistConfig = BlocklistConfig::deserialize(&value).expect("deserialize");

        assert_eq!(result.url, "https://example.com/blocklist.txt");
        assert!(result.load_on_start);
        assert_eq!(result.check_after_days, 4);
        assert_eq!(result.list_compression, "gzip");
        assert_eq!(result.list_type, "text");
        assert!((result.last_update - 1_700_000_000.0).abs() < f64::EPSILON);
        assert_eq!(result.list_size, 5000);
        assert_eq!(result.timeout, 30);
        assert_eq!(result.try_times, 3);
        assert_eq!(result.whitelisted, vec!["1.2.3.4"]);
    }

    #[test]
    fn when_blocklist_status_dict_then_fields_populate() {
        let value = make_dict(vec![
            ("state", RencodeValue::Str("Idle".into())),
            ("up_to_date", RencodeValue::Bool(true)),
            ("num_whited", RencodeValue::Int(10)),
            ("num_blocked", RencodeValue::Int(5000)),
            ("file_progress", RencodeValue::Float(1.0)),
            (
                "file_url",
                RencodeValue::Str("https://example.com/blocklist.txt".into()),
            ),
            ("file_size", RencodeValue::Int(1_048_576)),
            ("file_date", RencodeValue::Float(1_700_000_000.0)),
            ("file_type", RencodeValue::Str("p2p (gz)".into())),
            (
                "whitelisted",
                RencodeValue::List(vec![RencodeValue::Str("10.0.0.1".into())]),
            ),
        ]);

        let result: BlocklistStatus = BlocklistStatus::deserialize(&value).expect("deserialize");

        assert_eq!(result.state, "Idle");
        assert!(result.up_to_date);
        assert_eq!(result.num_whited, 10);
        assert_eq!(result.num_blocked, 5000);
        assert!((result.file_progress - 1.0).abs() < f64::EPSILON);
        assert_eq!(result.file_url, "https://example.com/blocklist.txt");
        assert_eq!(result.file_size, 1_048_576);
        assert!((result.file_date - 1_700_000_000.0).abs() < f64::EPSILON);
        assert_eq!(result.file_type, "p2p (gz)");
        assert_eq!(result.whitelisted, vec!["10.0.0.1"]);
    }
}
