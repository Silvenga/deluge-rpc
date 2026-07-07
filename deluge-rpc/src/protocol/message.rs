use crate::RencodeValue;
use crate::protocol::error::ProtocolError;
use crate::protocol::helpers::field_as_str;

const RPC_RESPONSE: i64 = 1;
const RPC_ERROR: i64 = 2;
const RPC_EVENT: i64 = 3;

/// A parsed RPC message from the Deluge daemon.
///
/// Discriminated by the type tag at position 0 of the wire tuple:
/// `1` = RPC_RESPONSE, `2` = RPC_ERROR, `3` = RPC_EVENT.
#[derive(Debug, Clone)]
pub enum DelugeRpcMessage {
    /// Successful RPC response (tag `1`).
    Response {
        /// Request ID matching the original request.
        id: u32,
        /// Return value of the RPC call.
        value: RencodeValue,
    },
    /// RPC error response (tag `2`).
    Error {
        /// Request ID matching the original request.
        id: u32,
        /// Python exception class name (e.g. `BadLoginError`).
        exc_type: String,
        /// Human-readable exception message.
        exc_msg: String,
        /// Python traceback string for debugging.
        traceback: String,
    },
    /// RPC event notification (tag `3`).
    Event {
        /// Event name (e.g. `TorrentAddedEvent`).
        name: String,
        /// Event arguments.
        args: Vec<RencodeValue>,
    },
}

impl DelugeRpcMessage {
    /// Parse a `DelugeRpcMessage` from a decoded rencode value.
    ///
    /// The value must be a list whose first element is the type tag
    /// (`1` = response, `2` = error, `3` = event).
    pub fn from_rencode_value(decoded: &RencodeValue) -> Result<Self, ProtocolError> {
        let inner = match decoded {
            RencodeValue::List(items) => items,
            other => return Err(ProtocolError::InvalidEnvelope(other.clone())),
        };

        if inner.is_empty() {
            return Err(ProtocolError::EmptyMessage);
        }

        let msg_type = match inner.first() {
            Some(RencodeValue::Int(t)) => *t,
            Some(other) => return Err(ProtocolError::InvalidTypeTag(other.clone())),
            None => return Err(ProtocolError::MissingTypeTag),
        };

        match msg_type {
            RPC_RESPONSE => {
                let id = match inner.get(1) {
                    Some(RencodeValue::Int(i)) => {
                        u32::try_from(*i).map_err(|_| ProtocolError::ResponseIdOutOfRange(*i))?
                    }
                    Some(other) => return Err(ProtocolError::InvalidResponseId(other.clone())),
                    None => return Err(ProtocolError::MissingResponseId),
                };
                let value = inner
                    .get(2)
                    .cloned()
                    .ok_or(ProtocolError::MissingReturnValue)?;
                Ok(Self::Response { id, value })
            }
            RPC_ERROR => {
                let id = match inner.get(1) {
                    Some(RencodeValue::Int(i)) => {
                        u32::try_from(*i).map_err(|_| ProtocolError::ErrorIdOutOfRange(*i))?
                    }
                    Some(other) => return Err(ProtocolError::InvalidErrorId(other.clone())),
                    None => return Err(ProtocolError::MissingErrorId),
                };
                let exc_type = field_as_str(inner.get(2)).unwrap_or_else(|| "<unknown>".to_owned());
                let exc_msg = match inner.get(3) {
                    Some(RencodeValue::List(args)) if !args.is_empty() => {
                        field_as_str(args.first()).unwrap_or_else(|| "<empty args>".to_owned())
                    }
                    _ => field_as_str(inner.get(3)).unwrap_or_else(|| "<unknown>".to_owned()),
                };
                let traceback = field_as_str(inner.get(5)).unwrap_or_else(|| "<none>".to_owned());
                Ok(Self::Error {
                    id,
                    exc_type,
                    exc_msg,
                    traceback,
                })
            }
            RPC_EVENT => {
                let name = field_as_str(inner.get(1)).unwrap_or_else(|| "<unknown>".to_owned());
                let args = match inner.get(2) {
                    Some(RencodeValue::List(items)) => items.clone(),
                    _ => Vec::new(),
                };
                Ok(Self::Event { name, args })
            }
            other => Err(ProtocolError::UnknownMessageType(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn when_response_message_then_id_and_value_extracted() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_RESPONSE),
            RencodeValue::Int(5),
            RencodeValue::Int(10),
        ]);

        let msg = DelugeRpcMessage::from_rencode_value(&message).expect("decode");
        match msg {
            DelugeRpcMessage::Response { id, value } => {
                assert_eq!(id, 5);
                assert_eq!(value, RencodeValue::Int(10));
            }
            other => panic!("expected Response, got {other:?}"),
        }
    }

    #[test]
    fn when_error_message_then_fields_extracted() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_ERROR),
            RencodeValue::Int(7),
            RencodeValue::Str(String::from("BadLoginError")),
            RencodeValue::List(vec![RencodeValue::Str(String::from("bad password"))]),
            RencodeValue::Dict(BTreeMap::new()),
            RencodeValue::Str(String::from("traceback here")),
        ]);

        let msg = DelugeRpcMessage::from_rencode_value(&message).expect("decode");
        match msg {
            DelugeRpcMessage::Error {
                id,
                exc_type,
                exc_msg,
                traceback,
            } => {
                assert_eq!(id, 7);
                assert_eq!(exc_type, "BadLoginError");
                assert_eq!(exc_msg, "bad password");
                assert_eq!(traceback, "traceback here");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn when_event_message_then_name_and_args_extracted() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_EVENT),
            RencodeValue::Str(String::from("TorrentAddedEvent")),
            RencodeValue::List(vec![RencodeValue::Int(123)]),
        ]);

        let msg = DelugeRpcMessage::from_rencode_value(&message).expect("decode");
        match msg {
            DelugeRpcMessage::Event { name, args } => {
                assert_eq!(name, "TorrentAddedEvent");
                assert_eq!(args, vec![RencodeValue::Int(123)]);
            }
            other => panic!("expected Event, got {other:?}"),
        }
    }

    #[test]
    fn when_event_with_no_args_then_empty_list() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_EVENT),
            RencodeValue::Str(String::from("ConfigValueChanged")),
        ]);

        let msg = DelugeRpcMessage::from_rencode_value(&message).expect("decode");
        match msg {
            DelugeRpcMessage::Event { args, .. } => assert!(args.is_empty()),
            other => panic!("expected Event, got {other:?}"),
        }
    }

    #[test]
    fn when_unknown_type_then_error() {
        let message = RencodeValue::List(vec![RencodeValue::Int(99)]);

        let result = DelugeRpcMessage::from_rencode_value(&message);
        assert!(result.is_err());
    }

    #[test]
    fn when_empty_list_envelope_then_error() {
        let message = RencodeValue::List(vec![]);
        let result = DelugeRpcMessage::from_rencode_value(&message);
        assert!(result.is_err());
    }
}
