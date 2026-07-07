use crate::cassette::interaction::Interaction;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, io};

/// A collection of recorded RPC interactions saved to a JSON file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cassette {
    /// The cassette format version.
    pub version: u32,
    /// ISO 8601 timestamp of when the cassette was recorded.
    pub recorded_at: String,
    /// The Deluge daemon version at recording time, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daemon_version: Option<String>,
    /// The recorded request-response interactions.
    pub interactions: Vec<Interaction>,
}

impl Cassette {
    /// Load a cassette from a JSON file on disk.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CassetteError> {
        let data = fs::read_to_string(path)?;
        Self::from_json_str(&data)
    }

    /// Save the cassette as a JSON file at the given path.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), CassetteError> {
        let json = self.to_json_string()?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Deserialize a cassette from a JSON string.
    pub fn from_json_str(s: &str) -> Result<Self, CassetteError> {
        Ok(serde_json::from_str(s)?)
    }

    /// Serialize the cassette to a pretty-printed JSON string.
    pub fn to_json_string(&self) -> Result<String, CassetteError> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

/// Errors that can occur when loading or saving a cassette.
#[derive(Debug, thiserror::Error)]
pub enum CassetteError {
    /// An I/O error occurred while reading or writing the file.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// The cassette JSON could not be parsed or serialized.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cassette::interaction::{Interaction, InteractionRequest, InteractionResponse};
    use deluge_rpc::RencodeValue;
    use std::collections::BTreeMap;
    use std::{env, process};

    fn make_cassette() -> Cassette {
        Cassette {
            version: 1,
            recorded_at: "2026-07-04T12:00:00Z".into(),
            daemon_version: Some("2.1.1".into()),
            interactions: vec![Interaction {
                request: InteractionRequest {
                    method: "core.get_free_space".into(),
                    args: RencodeValue::List(vec![RencodeValue::None]),
                    kwargs: RencodeValue::Dict(BTreeMap::new()),
                },
                response: InteractionResponse::Error {
                    exc_type: "NotEnoughSpace".into(),
                    exc_msg: "disk full".into(),
                    traceback: String::new(),
                },
            }],
        }
    }

    #[test]
    fn when_cassette_roundtrip_json_then_equal() {
        let original = make_cassette();

        let json = original.to_json_string().expect("serialize");
        let roundtripped = Cassette::from_json_str(&json).expect("deserialize");

        assert_eq!(original, roundtripped);
    }

    #[test]
    fn when_cassette_save_load_then_equal() {
        let original = make_cassette();
        let dir = env::temp_dir().join(format!("deluge-rpc-mock-test-{}", process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("cassette.json");

        original.save(&path).expect("save");
        let loaded = Cassette::load(&path).expect("load");

        assert_eq!(original, loaded);

        let _ = fs::remove_dir_all(&dir);
    }
}
