use crate::rencode::RencodeValue;
use anyhow::anyhow;
use std::collections::BTreeMap;

pub const RPC_RESPONSE: i64 = 1;
pub const RPC_ERROR: i64 = 2;
pub const RPC_EVENT: i64 = 3;

#[derive(Debug)]
pub enum ResponseOutcome {
    Return(RencodeValue),
    Continue,
}

pub fn handle_response(
    decoded: &RencodeValue,
    expected_id: u32,
    method: &str,
) -> anyhow::Result<ResponseOutcome> {
    let outer = match decoded {
        RencodeValue::List(items) if items.len() == 1 => {
            items.first().expect("len == 1 checked above")
        }
        other => {
            return Err(anyhow!(
                "unexpected RPC envelope shape (not a 1-element list): {other:?}"
            ));
        }
    };

    let inner = match outer {
        RencodeValue::List(parts) => parts,
        other => {
            return Err(anyhow!(
                "unexpected RPC message shape (not a list): {other:?}"
            ));
        }
    };

    if inner.is_empty() {
        return Err(anyhow!("empty RPC message"));
    }

    let msg_type = match inner.first() {
        Some(RencodeValue::Int(t)) => *t,
        Some(other) => return Err(anyhow!("RPC message type tag is not an int: {other:?}")),
        None => return Err(anyhow!("RPC message missing type tag")),
    };

    match msg_type {
        RPC_RESPONSE => extract_response_value(inner, expected_id, method),
        RPC_ERROR => Err(rpc_error(inner)),
        RPC_EVENT => {
            let event_name = field_as_str(inner.get(1)).unwrap_or_else(|| "<unknown>".to_owned());
            tracing::trace!(event = %event_name, "daemon RPC event");
            Ok(ResponseOutcome::Continue)
        }
        other => Err(anyhow!(
            "unexpected RPC message type {other} (expected 1/2/3)"
        )),
    }
}

fn extract_response_value(
    inner: &[RencodeValue],
    expected_id: u32,
    method: &str,
) -> anyhow::Result<ResponseOutcome> {
    let resp_id = match inner.get(1) {
        Some(RencodeValue::Int(i)) => *i,
        Some(other) => return Err(anyhow!("RPC response id is not an int: {other:?}")),
        None => return Err(anyhow!("RPC response missing id")),
    };
    if resp_id != i64::from(expected_id) {
        tracing::trace!(
            resp_id,
            expected = expected_id,
            "ignoring RPC response with mismatched id"
        );
        return Ok(ResponseOutcome::Continue);
    }
    let value = inner
        .get(2)
        .cloned()
        .ok_or_else(|| anyhow!("RPC response for `{method}` missing return value list"))?;
    Ok(ResponseOutcome::Return(value))
}

fn rpc_error(inner: &[RencodeValue]) -> anyhow::Error {
    let exc_type = field_as_str(inner.get(1)).unwrap_or_else(|| "<unknown>".to_owned());
    let exc_msg = field_as_str(inner.get(2)).unwrap_or_else(|| "<unknown>".to_owned());
    let traceback = field_as_str(inner.get(3)).unwrap_or_else(|| "<none>".to_owned());
    anyhow!("daemon RPC error ({exc_type}): {exc_msg}\ntraceback: {traceback}")
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

pub fn field_as_str(value: Option<&RencodeValue>) -> Option<String> {
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
    use crate::protocol::DelugeRpcRequest;

    fn unwrap_request(value: &RencodeValue) -> Vec<RencodeValue> {
        let outer = match value {
            RencodeValue::List(items) if items.len() == 1 => &items[0],
            _ => panic!("expected 1-element outer list"),
        };
        match outer {
            RencodeValue::List(p) => p.clone(),
            _ => panic!("expected inner list"),
        }
    }

    fn response_list(value: &RencodeValue) -> Vec<RencodeValue> {
        match value {
            RencodeValue::List(p) => p.clone(),
            _ => panic!("expected list"),
        }
    }

    #[test]
    fn when_login_request_built_then_envelope_has_correct_shape() {
        let mut kwargs = BTreeMap::new();
        kwargs.insert(
            RencodeValue::Str(String::from("client_version")),
            RencodeValue::Str(String::from("deluge-retain/0.1.0")),
        );
        let args = vec![
            RencodeValue::Str(String::from("localclient")),
            RencodeValue::Str(String::from("secret")),
        ];

        let request = DelugeRpcRequest::new("daemon.login")
            .with_args(args)
            .with_kwargs(kwargs)
            .into_rencode_value(1);

        let outer = match &request {
            RencodeValue::List(items) if items.len() == 1 => &items[0],
            _ => panic!("expected 1-element outer list"),
        };
        let parts = match outer {
            RencodeValue::List(p) => p,
            _ => panic!("expected inner list"),
        };
        assert_eq!(parts.len(), 4, "request tuple has 4 elements");
        assert_eq!(parts[0], RencodeValue::Int(1));
        assert_eq!(parts[1], RencodeValue::Str(String::from("daemon.login")));
        match &parts[2] {
            RencodeValue::List(args_inner) => {
                assert_eq!(args_inner.len(), 2);
                assert_eq!(
                    args_inner[0],
                    RencodeValue::Str(String::from("localclient"))
                );
                assert_eq!(args_inner[1], RencodeValue::Str(String::from("secret")));
            }
            _ => panic!("args is not a list"),
        }
        match &parts[3] {
            RencodeValue::Dict(map) => {
                assert_eq!(map.len(), 1);
                assert_eq!(
                    map.get(&RencodeValue::Str(String::from("client_version"))),
                    Some(&RencodeValue::Str(String::from("deluge-retain/0.1.0")))
                );
            }
            _ => panic!("kwargs is not a dict"),
        }
    }

    #[test]
    fn when_get_free_space_request_built_then_args_has_none() {
        let request = DelugeRpcRequest::new("core.get_free_space").into_rencode_value(2);
        let parts = unwrap_request(&request);
        assert_eq!(parts[0], RencodeValue::Int(2));
        match &parts[2] {
            RencodeValue::List(args) => {
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], RencodeValue::None);
            }
            _ => panic!("args is not a list"),
        }
        match &parts[3] {
            RencodeValue::Dict(map) => assert!(map.is_empty()),
            _ => panic!("kwargs is not a dict"),
        }
    }

    #[test]
    fn when_get_torrents_status_request_built_then_filter_dict_is_first() {
        let keys = vec![RencodeValue::Str(String::from("name"))];
        let args = vec![
            RencodeValue::Dict(BTreeMap::new()),
            RencodeValue::List(keys),
        ];
        let request = DelugeRpcRequest::new("core.get_torrents_status")
            .with_args(args)
            .into_rencode_value(3);
        let parts = unwrap_request(&request);
        match &parts[2] {
            RencodeValue::List(args) => {
                assert_eq!(args.len(), 2);
                assert!(
                    matches!(&args[0], RencodeValue::Dict(d) if d.is_empty()),
                    "filter_dict must be FIRST and empty"
                );
                assert!(
                    matches!(&args[1], RencodeValue::List(_)),
                    "keys must be SECOND"
                );
            }
            _ => panic!("args is not a list"),
        }
    }

    #[test]
    fn when_remove_torrent_request_built_then_args_has_id_and_true() {
        let args = vec![
            RencodeValue::Str(String::from("deadbeef")),
            RencodeValue::Bool(true),
        ];
        let request = DelugeRpcRequest::new("core.remove_torrent")
            .with_args(args)
            .into_rencode_value(4);
        let parts = unwrap_request(&request);
        match &parts[2] {
            RencodeValue::List(args) => {
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], RencodeValue::Str(String::from("deadbeef")));
                assert_eq!(args[1], RencodeValue::Bool(true));
            }
            _ => panic!("args is not a list"),
        }
    }

    #[test]
    fn when_response_is_success_then_return_value_is_extracted() {
        let response = RencodeValue::List(vec![RencodeValue::Int(1_073_741_824)]);

        let bytes = extract_single_int(&response, "core.get_free_space").expect("extract");
        assert_eq!(bytes, 1_073_741_824);
    }

    #[test]
    fn when_response_is_error_then_message_is_extractable() {
        let response = RencodeValue::List(vec![
            RencodeValue::Int(RPC_ERROR),
            RencodeValue::Str(String::from("BadLoginError")),
            RencodeValue::Str(String::from("bad password")),
            RencodeValue::Str(String::from("traceback here")),
        ]);

        let parts = response_list(&response);
        let exc_type = field_as_str(parts.get(1)).unwrap();
        let exc_msg = field_as_str(parts.get(2)).unwrap();
        assert_eq!(exc_type, "BadLoginError");
        assert_eq!(exc_msg, "bad password");
    }

    #[test]
    fn when_response_is_event_then_type_tag_is_3() {
        let response = RencodeValue::List(vec![
            RencodeValue::Int(RPC_EVENT),
            RencodeValue::Str(String::from("TorrentAddedEvent")),
            RencodeValue::List(vec![]),
        ]);
        let parts = response_list(&response);
        assert_eq!(parts[0], RencodeValue::Int(RPC_EVENT));
    }

    #[test]
    fn when_remove_torrent_response_then_bool_is_extracted() {
        let response = RencodeValue::List(vec![RencodeValue::Bool(true)]);
        let value = extract_single(&response, "core.remove_torrent").expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[test]
    fn when_free_space_negative_then_get_free_space_returns_error() {
        let response = RencodeValue::List(vec![RencodeValue::Int(-1)]);
        let result = extract_single_int(&response, "core.get_free_space");
        let bytes = result.expect("extract succeeds");
        assert!(bytes < 0);
    }

    #[test]
    fn when_handle_response_with_matching_id_then_returns_value() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_RESPONSE),
            RencodeValue::Int(5),
            RencodeValue::List(vec![RencodeValue::Int(42)]),
        ]);
        let response = RencodeValue::List(vec![message]);

        let outcome = handle_response(&response, 5, "test").expect("ok");
        match outcome {
            ResponseOutcome::Return(RencodeValue::List(items)) => {
                assert_eq!(items, vec![RencodeValue::Int(42)]);
            }
            other => panic!("expected Return, got {other:?}"),
        }
    }

    #[test]
    fn when_handle_response_with_mismatched_id_then_continues() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_RESPONSE),
            RencodeValue::Int(99),
            RencodeValue::List(vec![RencodeValue::Int(42)]),
        ]);
        let response = RencodeValue::List(vec![message]);

        let outcome = handle_response(&response, 5, "test").expect("ok");
        assert!(matches!(outcome, ResponseOutcome::Continue));
    }

    #[test]
    fn when_handle_response_with_event_then_continues() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_EVENT),
            RencodeValue::Str(String::from("TorrentAddedEvent")),
            RencodeValue::List(vec![]),
        ]);
        let response = RencodeValue::List(vec![message]);

        let outcome = handle_response(&response, 5, "test").expect("ok");
        assert!(matches!(outcome, ResponseOutcome::Continue));
    }

    #[test]
    fn when_handle_response_with_error_then_returns_error() {
        let message = RencodeValue::List(vec![
            RencodeValue::Int(RPC_ERROR),
            RencodeValue::Str(String::from("BadLoginError")),
            RencodeValue::Str(String::from("bad password")),
            RencodeValue::Str(String::from("tb")),
        ]);
        let response = RencodeValue::List(vec![message]);

        let err = handle_response(&response, 5, "test").expect_err("should error");
        let msg = err.to_string();
        assert!(msg.contains("BadLoginError"), "got: {msg}");
        assert!(msg.contains("bad password"), "got: {msg}");
    }
}
