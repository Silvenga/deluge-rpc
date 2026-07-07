use crate::RencodeValue;
use crate::protocol::error::ProtocolError;
use std::collections::BTreeMap;

/// Extracts the return value from an RPC response.
pub fn extract_single(value: &RencodeValue) -> Result<RencodeValue, ProtocolError> {
    Ok(value.clone())
}

/// Extracts an `i64` return value from an RPC response.
pub fn extract_single_int(value: &RencodeValue, method: &str) -> Result<i64, ProtocolError> {
    let single = extract_single(value)?;
    match single {
        RencodeValue::Int(i) => Ok(i),
        other => Err(ProtocolError::NotInt {
            method: method.to_owned(),
            value: other,
        }),
    }
}

/// Extracts a dict reference from an RPC response.
pub fn extract_single_dict<'a>(
    value: &'a RencodeValue,
    method: &str,
) -> Result<&'a BTreeMap<RencodeValue, RencodeValue>, ProtocolError> {
    match value {
        RencodeValue::Dict(map) => Ok(map),
        other => Err(ProtocolError::NotDict {
            method: method.to_owned(),
            value: other.clone(),
        }),
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

    #[test]
    fn when_extract_single_int_then_returns_value() {
        let response = RencodeValue::Int(1_073_741_824);
        let bytes = extract_single_int(&response, "core.get_free_space").expect("extract");
        assert_eq!(bytes, 1_073_741_824);
    }

    #[test]
    fn when_extract_single_bool_then_returns_value() {
        let response = RencodeValue::Bool(true);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[test]
    fn when_extract_single_str_then_returns_value() {
        let response = RencodeValue::Str("2.1.1".into());
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Str(s) => assert_eq!(s, "2.1.1"),
            other => panic!("expected str, got {other:?}"),
        }
    }

    #[test]
    fn when_extract_single_dict_then_returns_dict() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("key".into()),
            RencodeValue::Str("value".into()),
        );
        let response = RencodeValue::Dict(map.clone());

        let result = extract_single_dict(&response, "core.get_torrents_status").expect("extract");
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.get(&RencodeValue::Str("key".into())),
            Some(&RencodeValue::Str("value".into()))
        );
    }
}
