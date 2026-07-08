use deluge_rpc_client::{RencodeValue, from_json, to_json};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// A recorded request-response pair from a Deluge RPC interaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interaction {
    /// The request sent to the daemon.
    pub request: InteractionRequest,
    /// The response received from the daemon.
    pub response: InteractionResponse,
}

/// The request half of a recorded interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionRequest {
    /// The RPC method name (e.g. `"daemon.info"`).
    pub method: String,
    /// The positional arguments sent to the method.
    pub args: RencodeValue,
    /// The keyword arguments sent to the method.
    pub kwargs: RencodeValue,
}

/// The response half of a recorded interaction, either success or error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionResponse {
    /// A successful response with the returned value.
    Ok {
        /// The value returned by the RPC method.
        value: RencodeValue,
    },
    /// An error response from the daemon.
    Error {
        /// The Python exception type name.
        exc_type: String,
        /// The exception message.
        exc_msg: String,
        /// The Python traceback string.
        traceback: String,
    },
}

impl Serialize for InteractionRequest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Request", 3)?;
        state.serialize_field("method", &self.method)?;
        state.serialize_field("args", &to_json(&self.args))?;
        state.serialize_field("kwargs", &to_json(&self.kwargs))?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for InteractionRequest {
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
        Ok(InteractionRequest {
            method: method.to_owned(),
            args: from_json(args_json).map_err(de::Error::custom)?,
            kwargs: from_json(kwargs_json).map_err(de::Error::custom)?,
        })
    }
}

impl Serialize for InteractionResponse {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            InteractionResponse::Ok { value } => {
                let mut state = serializer.serialize_struct("Response", 2)?;
                state.serialize_field("type", "ok")?;
                state.serialize_field("value", &to_json(value))?;
                state.end()
            }
            InteractionResponse::Error {
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

impl<'de> Deserialize<'de> for InteractionResponse {
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
                Ok(InteractionResponse::Ok {
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
                Ok(InteractionResponse::Error {
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
    use deluge_rpc_client::RencodeValue;

    #[test]
    fn when_response_error_then_roundtrip_preserves_fields() {
        let response = InteractionResponse::Error {
            exc_type: "BadLoginError".into(),
            exc_msg: "bad password".into(),
            traceback: "line 1\nline 2".into(),
        };
        let json = serde_json::to_string(&response).expect("serialize");
        let roundtripped: InteractionResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(response, roundtripped);
    }

    #[test]
    fn when_response_ok_then_roundtrip_preserves_value() {
        let response = InteractionResponse::Ok {
            value: RencodeValue::Str("hello".into()),
        };
        let json = serde_json::to_string(&response).expect("serialize");
        let roundtripped: InteractionResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(response, roundtripped);
    }
}
