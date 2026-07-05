use deluge_rpc::RencodeValue;
use serde::de;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs;
use std::path::Path;

pub fn write_cassette_atomic(path: &Path, cassette: &Cassette) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(cassette)
        .map_err(|e| anyhow::anyhow!("failed to serialize cassette: {e}"))?;

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, json)
        .map_err(|e| anyhow::anyhow!("failed to write temporary cassette: {e}"))?;

    fs::rename(&temp_path, path)
        .map_err(|e| anyhow::anyhow!("failed to rename temporary cassette: {e}"))?;

    Ok(())
}

pub fn load_cassette(path: &Path) -> anyhow::Result<Cassette> {
    let data =
        fs::read_to_string(path).map_err(|e| anyhow::anyhow!("failed to read cassette: {e}"))?;
    serde_json::from_str(&data).map_err(|e| anyhow::anyhow!("failed to parse cassette: {e}"))
}

pub fn response_to_tagged_json(value: &RencodeValue) -> serde_json::Value {
    deluge_rpc::to_json(value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cassette {
    pub version: u32,
    pub recorded_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daemon_version: Option<String>,
    pub interactions: Vec<Interaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub request: Request,
    pub response: Response,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub args: RencodeValue,
    pub kwargs: RencodeValue,
}

#[derive(Debug, Clone)]
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
        state.serialize_field("args", &deluge_rpc::to_json(&self.args))?;
        state.serialize_field("kwargs", &deluge_rpc::to_json(&self.kwargs))?;
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
            args: deluge_rpc::from_json(args_json).map_err(de::Error::custom)?,
            kwargs: deluge_rpc::from_json(kwargs_json).map_err(de::Error::custom)?,
        })
    }
}

impl Serialize for Response {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Response::Ok { value } => {
                let mut state = serializer.serialize_struct("Response", 2)?;
                state.serialize_field("type", "ok")?;
                state.serialize_field("value", &deluge_rpc::to_json(value))?;
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
                    value: deluge_rpc::from_json(value).map_err(de::Error::custom)?,
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
