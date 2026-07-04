use crate::rencode::RencodeValue;
use anyhow::{anyhow, bail};
use std::collections::BTreeMap;

const RPC_RESPONSE: i64 = 1;
const RPC_ERROR: i64 = 2;
const RPC_EVENT: i64 = 3;

#[derive(Debug, Clone)]
pub enum DelugeRpcMessage {
    Response {
        id: u32,
        value: RencodeValue,
    },
    Error {
        id: u32,
        exc_type: String,
        exc_msg: String,
        traceback: String,
    },
    Event {
        name: String,
        args: Vec<RencodeValue>,
    },
}

pub fn decode_message(decoded: &RencodeValue) -> anyhow::Result<DelugeRpcMessage> {
    let outer = match decoded {
        RencodeValue::List(items) if items.len() == 1 => {
            items.first().expect("len == 1 checked above")
        }
        other => bail!("unexpected RPC envelope shape (not a 1-element list): {other:?}"),
    };

    let inner = match outer {
        RencodeValue::List(parts) => parts,
        other => bail!("unexpected RPC message shape (not a list): {other:?}"),
    };

    if inner.is_empty() {
        bail!("empty RPC message");
    }

    let msg_type = match inner.first() {
        Some(RencodeValue::Int(t)) => *t,
        Some(other) => bail!("RPC message type tag is not an int: {other:?}"),
        None => bail!("RPC message missing type tag"),
    };

    match msg_type {
        RPC_RESPONSE => {
            let id = match inner.get(1) {
                Some(RencodeValue::Int(i)) => u32::try_from(*i)
                    .map_err(|_| anyhow!("RPC response id out of u32 range: {i}"))?,
                Some(other) => bail!("RPC response id is not an int: {other:?}"),
                None => bail!("RPC response missing id"),
            };
            let value = inner
                .get(2)
                .cloned()
                .ok_or_else(|| anyhow!("RPC response missing return value"))?;
            Ok(DelugeRpcMessage::Response { id, value })
        }
        RPC_ERROR => {
            let id = match inner.get(1) {
                Some(RencodeValue::Int(i)) => {
                    u32::try_from(*i).map_err(|_| anyhow!("RPC error id out of u32 range: {i}"))?
                }
                Some(other) => bail!("RPC error id is not an int: {other:?}"),
                None => bail!("RPC error missing id"),
            };
            let exc_type = field_as_str(inner.get(2)).unwrap_or_else(|| "<unknown>".to_owned());
            let exc_msg = match inner.get(3) {
                Some(RencodeValue::List(args)) if !args.is_empty() => {
                    field_as_str(args.first()).unwrap_or_else(|| "<empty args>".to_owned())
                }
                _ => field_as_str(inner.get(3)).unwrap_or_else(|| "<unknown>".to_owned()),
            };
            let traceback = field_as_str(inner.get(5)).unwrap_or_else(|| "<none>".to_owned());
            Ok(DelugeRpcMessage::Error {
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
            Ok(DelugeRpcMessage::Event { name, args })
        }
        other => bail!("unexpected RPC message type {other} (expected 1/2/3)"),
    }
}

pub fn extract_single(value: &RencodeValue, method: &str) -> anyhow::Result<RencodeValue> {
    match value {
        RencodeValue::List(items) if items.len() == 1 => {
            Ok(items.first().expect("len == 1 checked above").clone())
        }
        other => Err(anyhow!(
            "{method} returned unexpected return shape (expected 1-element list): {other:?}"
        )),
    }
}

pub fn extract_single_int(value: &RencodeValue, method: &str) -> anyhow::Result<i64> {
    let single = extract_single(value, method)?;
    match single {
        RencodeValue::Int(i) => Ok(i),
        other => Err(anyhow!("{method} returned non-int value: {other:?}")),
    }
}

pub fn extract_single_dict<'a>(
    value: &'a RencodeValue,
    method: &str,
) -> anyhow::Result<&'a BTreeMap<RencodeValue, RencodeValue>> {
    match value {
        RencodeValue::List(items) if items.len() == 1 => match items.first() {
            Some(RencodeValue::Dict(map)) => Ok(map),
            Some(other) => Err(anyhow!("{method} returned non-dict value: {other:?}")),
            None => Err(anyhow!("{method} returned empty return list")),
        },
        other => Err(anyhow!(
            "{method} returned unexpected return shape (expected 1-element list): {other:?}"
        )),
    }
}

fn field_as_str(value: Option<&RencodeValue>) -> Option<String> {
    match value? {
        RencodeValue::Str(s) => Some(s.clone()),
        RencodeValue::Bytes(b) => String::from_utf8(b.clone()).ok(),
        RencodeValue::Int(i) => Some(i.to_string()),
        other => Some(format!("{other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_response_message_then_id_and_value_extracted() {
        let message = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Int(RPC_RESPONSE),
            RencodeValue::Int(5),
            RencodeValue::List(vec![RencodeValue::Int(42)]),
        ])]);

        let msg = decode_message(&message).expect("decode");
        match msg {
            DelugeRpcMessage::Response { id, value } => {
                assert_eq!(id, 5);
                match value {
                    RencodeValue::List(items) => assert_eq!(items, vec![RencodeValue::Int(42)]),
                    other => panic!("expected list value, got {other:?}"),
                }
            }
            other => panic!("expected Response, got {other:?}"),
        }
    }

    #[test]
    fn when_error_message_then_fields_extracted() {
        let message = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Int(RPC_ERROR),
            RencodeValue::Int(7),
            RencodeValue::Str(String::from("BadLoginError")),
            RencodeValue::List(vec![RencodeValue::Str(String::from("bad password"))]),
            RencodeValue::Dict(BTreeMap::new()),
            RencodeValue::Str(String::from("traceback here")),
        ])]);

        let msg = decode_message(&message).expect("decode");
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
        let message = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Int(RPC_EVENT),
            RencodeValue::Str(String::from("TorrentAddedEvent")),
            RencodeValue::List(vec![RencodeValue::Int(123)]),
        ])]);

        let msg = decode_message(&message).expect("decode");
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
        let message = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Int(RPC_EVENT),
            RencodeValue::Str(String::from("ConfigValueChanged")),
        ])]);

        let msg = decode_message(&message).expect("decode");
        match msg {
            DelugeRpcMessage::Event { args, .. } => assert!(args.is_empty()),
            other => panic!("expected Event, got {other:?}"),
        }
    }

    #[test]
    fn when_unknown_type_then_error() {
        let message = RencodeValue::List(vec![RencodeValue::List(vec![RencodeValue::Int(99)])]);

        let result = decode_message(&message);
        assert!(result.is_err());
    }

    #[test]
    fn when_extract_single_int_then_returns_value() {
        let response = RencodeValue::List(vec![RencodeValue::Int(1_073_741_824)]);
        let bytes = extract_single_int(&response, "core.get_free_space").expect("extract");
        assert_eq!(bytes, 1_073_741_824);
    }

    #[test]
    fn when_extract_single_bool_then_returns_value() {
        let response = RencodeValue::List(vec![RencodeValue::Bool(true)]);
        let value = extract_single(&response, "core.remove_torrent").expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }
}
