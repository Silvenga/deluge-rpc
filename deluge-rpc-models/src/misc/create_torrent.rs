use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::fmt;

/// Torrent metadata format for `core.create_torrent`.
///
/// Serializes to the lowercase string values accepted by the daemon
/// (`"v1"`, `"v2"`, `"hybrid"`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TorrentFormat {
    /// Bittorrent v1 metadata only.
    #[default]
    V1,
    /// Bittorrent v2 metadata only.
    V2,
    /// Both v1 and v2 metadata.
    Hybrid,
}

/// Request for `core.create_torrent`.
///
/// `path`, `tracker`, and `piece_length` are required and sent as positional
/// args. All other parameters mirror the daemon's keyword defaults and are
/// omitted from the request when unset. Serialization yields the kwargs dict
/// only; the required fields are skipped.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CreateTorrentRequest {
    /// Path to the file or directory on the daemon filesystem to create a torrent from.
    #[serde(skip)]
    pub path: String,
    /// Tracker URL.
    #[serde(skip)]
    pub tracker: String,
    /// Piece length in bytes.
    #[serde(skip)]
    pub piece_length: i64,
    /// Comment embedded in the torrent metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Output path on the daemon filesystem for the `.torrent` file. Defaults to `<path>.torrent`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Web seed URLs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webseeds: Option<Vec<String>>,
    /// Whether the torrent is private.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    /// `created by` field embedded in the torrent metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// Tiered tracker URLs, replacing `tracker`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trackers: Option<Vec<Vec<String>>>,
    /// Whether to add the created torrent to the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_to_session: Option<bool>,
    /// Torrent metadata format. Defaults to v1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub torrent_format: Option<TorrentFormat>,
    /// PEM-encoded CA certificate for SSL peers. Requires a daemon supporting
    /// the `ca_cert` argument; older daemons may reject it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_cert: Option<String>,
}

impl CreateTorrentRequest {
    /// Create a request with the required parameters.
    pub fn new(path: impl Into<String>, tracker: impl Into<String>, piece_length: i64) -> Self {
        Self {
            path: path.into(),
            tracker: tracker.into(),
            piece_length,
            comment: None,
            target: None,
            webseeds: None,
            private: None,
            created_by: None,
            trackers: None,
            add_to_session: None,
            torrent_format: None,
            ca_cert: None,
        }
    }

    /// Set the comment embedded in the torrent metadata.
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Set the output path for the `.torrent` file.
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Set the web seed URLs.
    pub fn with_webseeds(mut self, webseeds: Vec<String>) -> Self {
        self.webseeds = Some(webseeds);
        self
    }

    /// Set whether the torrent is private.
    pub fn with_private(mut self, private: bool) -> Self {
        self.private = Some(private);
        self
    }

    /// Set the `created by` field embedded in the torrent metadata.
    pub fn with_created_by(mut self, created_by: impl Into<String>) -> Self {
        self.created_by = Some(created_by.into());
        self
    }

    /// Set tiered tracker URLs, replacing `tracker`.
    pub fn with_trackers(mut self, trackers: Vec<Vec<String>>) -> Self {
        self.trackers = Some(trackers);
        self
    }

    /// Set whether to add the created torrent to the session.
    pub fn with_add_to_session(mut self, add_to_session: bool) -> Self {
        self.add_to_session = Some(add_to_session);
        self
    }

    /// Set the torrent metadata format.
    pub fn with_torrent_format(mut self, torrent_format: TorrentFormat) -> Self {
        self.torrent_format = Some(torrent_format);
        self
    }

    /// Set the PEM-encoded CA certificate for SSL peers.
    pub fn with_ca_cert(mut self, ca_cert: impl Into<String>) -> Self {
        self.ca_cert = Some(ca_cert.into());
        self
    }
}

/// Result of `core.create_torrent()`.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTorrentResult {
    /// Torrent filename.
    pub filename: String,
    /// Base64-encoded bencoded torrent data.
    pub file_dump: String,
}

impl<'de> Deserialize<'de> for CreateTorrentResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CreateVisitor;

        impl<'de> Visitor<'de> for CreateVisitor {
            type Value = CreateTorrentResult;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 2-element tuple (torrent_id, filedump)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let filename: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let file_dump: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(CreateTorrentResult {
                    filename,
                    file_dump,
                })
            }
        }

        deserializer.deserialize_seq(CreateVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rpc_rencode::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_create_torrent_result_from_tuple_then_fields_populate() {
        let value = RencodeValue::List(vec![
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
            RencodeValue::Str("base64-encoded-data".into()),
        ]);

        let result: CreateTorrentResult =
            CreateTorrentResult::deserialize(&value).expect("deserialize");

        assert_eq!(result.filename, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
        assert_eq!(result.file_dump, "base64-encoded-data");
    }

    #[test]
    fn when_create_torrent_request_default_then_optional_fields_none() {
        let request =
            CreateTorrentRequest::new("/data/file.txt", "http://tracker/announce", 262144);

        assert_eq!(request.path, "/data/file.txt");
        assert_eq!(request.tracker, "http://tracker/announce");
        assert_eq!(request.piece_length, 262144);
        assert_eq!(request.comment, None);
        assert_eq!(request.target, None);
        assert_eq!(request.webseeds, None);
        assert_eq!(request.private, None);
        assert_eq!(request.created_by, None);
        assert_eq!(request.trackers, None);
        assert_eq!(request.add_to_session, None);
        assert_eq!(request.torrent_format, None);
        assert_eq!(request.ca_cert, None);
    }

    #[test]
    fn when_create_torrent_request_serialized_then_omits_unset_fields() {
        let request =
            CreateTorrentRequest::new("/data/file.txt", "http://tracker/announce", 262144);

        let json = serde_json::to_value(&request).expect("serialize");

        assert_eq!(json.as_object().expect("object").len(), 0);
    }

    #[test]
    fn when_create_torrent_request_serialized_then_torrent_format_is_lowercase() {
        let request =
            CreateTorrentRequest::new("/data/file.txt", "http://tracker/announce", 262144)
                .with_torrent_format(TorrentFormat::Hybrid)
                .with_ca_cert("-----BEGIN CERTIFICATE-----");

        let json = serde_json::to_value(&request).expect("serialize");

        assert_eq!(json["torrent_format"], "hybrid");
        assert_eq!(json["ca_cert"], "-----BEGIN CERTIFICATE-----");
    }
}
