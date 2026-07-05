use deluge_rpc::RencodeValue;
use deluge_rpc::{from_json, to_json};
use serde::de;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum CassetteError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cassette {
    pub version: u32,
    pub recorded_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daemon_version: Option<String>,
    pub interactions: Vec<Interaction>,
}

impl Cassette {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CassetteError> {
        let data = fs::read_to_string(path)?;
        Self::from_json_str(&data)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), CassetteError> {
        let json = self.to_json_string()?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn from_json_str(s: &str) -> Result<Self, CassetteError> {
        Ok(serde_json::from_str(s)?)
    }

    pub fn to_json_string(&self) -> Result<String, CassetteError> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interaction {
    pub request: Request,
    pub response: Response,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub args: RencodeValue,
    pub kwargs: RencodeValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Ok {
        value: RencodeValue,
    },
    Error {
        exc_type: String,
        exc_msg: String,
        traceback: String,
    },
}

impl Serialize for Request {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Request", 3)?;
        state.serialize_field("method", &self.method)?;
        state.serialize_field("args", &to_json(&self.args))?;
        state.serialize_field("kwargs", &to_json(&self.kwargs))?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Request {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let json = serde_json::Value::deserialize(deserializer)?;
        let obj = json
            .as_object()
            .ok_or_else(|| de::Error::custom("expected object for Request"))?;
        let method = obj
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| de::Error::custom("missing field 'method'"))?;
        let args_json = obj
            .get("args")
            .ok_or_else(|| de::Error::custom("missing field 'args'"))?;
        let kwargs_json = obj
            .get("kwargs")
            .ok_or_else(|| de::Error::custom("missing field 'kwargs'"))?;
        Ok(Request {
            method: method.to_owned(),
            args: from_json(args_json).map_err(de::Error::custom)?,
            kwargs: from_json(kwargs_json).map_err(de::Error::custom)?,
        })
    }
}

impl Serialize for Response {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Response::Ok { value } => {
                let mut state = serializer.serialize_struct("Response", 2)?;
                state.serialize_field("type", "ok")?;
                state.serialize_field("value", &to_json(value))?;
                state.end()
            }
            Response::Error {
                exc_type,
                exc_msg,
                traceback,
            } => {
                let mut state = serializer.serialize_struct("Response", 4)?;
                state.serialize_field("type", "error")?;
                state.serialize_field("exc_type", exc_type)?;
                state.serialize_field("exc_msg", exc_msg)?;
                state.serialize_field("traceback", traceback)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let json = serde_json::Value::deserialize(deserializer)?;
        let obj = json
            .as_object()
            .ok_or_else(|| de::Error::custom("expected object for Response"))?;
        let type_str = obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| de::Error::custom("missing field 'type'"))?;
        match type_str {
            "ok" => {
                let value = obj
                    .get("value")
                    .ok_or_else(|| de::Error::custom("missing field 'value'"))?;
                Ok(Response::Ok {
                    value: from_json(value).map_err(de::Error::custom)?,
                })
            }
            "error" => {
                let exc_type = obj
                    .get("exc_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned();
                let exc_msg = obj
                    .get("exc_msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned();
                let traceback = obj
                    .get("traceback")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned();
                Ok(Response::Error {
                    exc_type,
                    exc_msg,
                    traceback,
                })
            }
            other => Err(de::Error::custom(format!("unknown response type: {other}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rpc::RencodeValue;
    use std::collections::BTreeMap;
    use std::env;
    use std::process;

    fn make_cassette() -> Cassette {
        Cassette {
            version: 1,
            recorded_at: "2026-07-04T12:00:00Z".into(),
            daemon_version: Some("2.1.1".into()),
            interactions: vec![Interaction {
                request: Request {
                    method: "core.get_free_space".into(),
                    args: RencodeValue::List(vec![RencodeValue::None]),
                    kwargs: RencodeValue::Dict(BTreeMap::new()),
                },
                response: Response::Error {
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

    #[test]
    fn when_response_error_then_roundtrip_preserves_fields() {
        let response = Response::Error {
            exc_type: "BadLoginError".into(),
            exc_msg: "bad password".into(),
            traceback: "line 1\nline 2".into(),
        };
        let json = serde_json::to_string(&response).expect("serialize");
        let roundtripped: Response = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(response, roundtripped);
    }

    #[test]
    fn when_response_ok_then_roundtrip_preserves_value() {
        let response = Response::Ok {
            value: RencodeValue::Str("hello".into()),
        };
        let json = serde_json::to_string(&response).expect("serialize");
        let roundtripped: Response = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(response, roundtripped);
    }
}
