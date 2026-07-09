use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the Notifications plugin.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NotificationsConfig {
    /// Whether email notifications are enabled.
    pub smtp_enabled: bool,
    /// SMTP server hostname.
    pub smtp_host: String,
    /// SMTP server port.
    pub smtp_port: i64,
    /// SMTP username.
    pub smtp_user: String,
    /// SMTP password.
    pub smtp_pass: String,
    /// From address for notification emails.
    pub smtp_from: String,
    /// Whether to use TLS for SMTP.
    pub smtp_tls: bool,
    /// Email addresses to notify.
    pub smtp_recipients: Vec<String>,
    /// Event subscriptions per notification type.
    pub subscriptions: HashMap<String, Vec<String>>,
}

/// An event that the Notifications plugin can handle.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HandledEvent {
    /// The event name (e.g. "TorrentFinishedEvent").
    pub event_name: String,
    /// The Python class docstring for the event.
    pub docstring: String,
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

    #[test]
    fn when_notifications_config_dict_then_fields_populate() {
        let mut subscriptions = BTreeMap::new();
        subscriptions.insert(
            RencodeValue::Str("email".into()),
            RencodeValue::List(vec![RencodeValue::Str("TorrentFinishedEvent".into())]),
        );

        let value = make_dict(vec![
            ("smtp_enabled", RencodeValue::Bool(true)),
            ("smtp_host", RencodeValue::Str("smtp.example.com".into())),
            ("smtp_port", RencodeValue::Int(587)),
            ("smtp_user", RencodeValue::Str("user".into())),
            ("smtp_pass", RencodeValue::Str("pass".into())),
            ("smtp_from", RencodeValue::Str("deluge@example.com".into())),
            ("smtp_tls", RencodeValue::Bool(true)),
            (
                "smtp_recipients",
                RencodeValue::List(vec![RencodeValue::Str("admin@example.com".into())]),
            ),
            ("subscriptions", RencodeValue::Dict(subscriptions)),
        ]);

        let result: NotificationsConfig =
            NotificationsConfig::deserialize(&value).expect("deserialize");

        assert!(result.smtp_enabled);
        assert_eq!(result.smtp_host, "smtp.example.com");
        assert_eq!(result.smtp_port, 587);
        assert_eq!(result.smtp_user, "user");
        assert_eq!(result.smtp_pass, "pass");
        assert_eq!(result.smtp_from, "deluge@example.com");
        assert!(result.smtp_tls);
        assert_eq!(result.smtp_recipients, vec!["admin@example.com"]);
        assert_eq!(
            result.subscriptions.get("email"),
            Some(&vec!["TorrentFinishedEvent".to_owned()])
        );
    }

    #[test]
    fn when_handled_event_tuple_then_fields_populate() {
        let value = RencodeValue::List(vec![
            RencodeValue::Str("TorrentFinishedEvent".into()),
            RencodeValue::Str("Emitted when a torrent finishes downloading.".into()),
        ]);

        let result: HandledEvent = HandledEvent::deserialize(&value).expect("deserialize");

        assert_eq!(result.event_name, "TorrentFinishedEvent");
        assert_eq!(
            result.docstring,
            "Emitted when a torrent finishes downloading."
        );
    }
}
