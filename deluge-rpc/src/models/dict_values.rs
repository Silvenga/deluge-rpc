use crate::rencode::RencodeError;
use crate::RencodeValue;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Extracts all key-value pairs from a `RencodeValue::Dict`, deserializing each value into `V`.
///
/// Keys must be `RencodeValue::Str` (cloned directly) or `RencodeValue::Bytes` (decoded as UTF-8).
/// Any other key type (`Int`, `Bool`, `Float`, `List`, `Dict`, `None`) returns an error.
pub fn deserialize_dict_values<'de, V: Deserialize<'de>>(
    value: &'de RencodeValue,
) -> Result<Vec<(String, V)>, RencodeError> {
    let map: &BTreeMap<RencodeValue, RencodeValue> = match value {
        RencodeValue::Dict(map) => map,
        other => {
            return Err(RencodeError::WrongType {
                field: String::new(),
                expected: "dict",
                got: variant_name(other),
            })
        }
    };

    let mut result = Vec::with_capacity(map.len());
    for (key, val) in map {
        let key_str = extract_key(key)?;
        let deserialized: V = V::deserialize(val)?;
        result.push((key_str, deserialized));
    }
    Ok(result)
}

fn extract_key(key: &RencodeValue) -> Result<String, RencodeError> {
    match key {
        RencodeValue::Str(s) => Ok(s.clone()),
        RencodeValue::Bytes(b) => String::from_utf8(b.clone())
            .map_err(|_| RencodeError::Message("dict key bytes are not valid UTF-8".into())),
        RencodeValue::Int(_) => Err(RencodeError::Message("int cannot be a dict key".into())),
        RencodeValue::Bool(_) => Err(RencodeError::Message("bool cannot be a dict key".into())),
        RencodeValue::Float(_) => Err(RencodeError::Message("float cannot be a dict key".into())),
        RencodeValue::List(_) => Err(RencodeError::Message("list cannot be a dict key".into())),
        RencodeValue::Dict(_) => Err(RencodeError::Message("dict cannot be a dict key".into())),
        RencodeValue::None => Err(RencodeError::Message("none cannot be a dict key".into())),
    }
}

fn variant_name(v: &RencodeValue) -> &'static str {
    match v {
        RencodeValue::None => "none",
        RencodeValue::Bool(_) => "bool",
        RencodeValue::Int(_) => "int",
        RencodeValue::Str(_) => "str",
        RencodeValue::Bytes(_) => "bytes",
        RencodeValue::List(_) => "list",
        RencodeValue::Dict(_) => "dict",
        RencodeValue::Float(_) => "float",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestValue {
        name: String,
        count: i64,
    }

    fn make_dict_with_i64_values() -> RencodeValue {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("a".into()), RencodeValue::Int(1));
        map.insert(RencodeValue::Str("b".into()), RencodeValue::Int(2));
        map.insert(RencodeValue::Str("c".into()), RencodeValue::Int(3));
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_dict_has_string_keys_then_values_deserialized() {
        let value = make_dict_with_i64_values();

        let result: Vec<(String, i64)> = deserialize_dict_values(&value).expect("deserialize");

        assert_eq!(result.len(), 3);
        assert!(result.contains(&("a".into(), 1)));
        assert!(result.contains(&("b".into(), 2)));
        assert!(result.contains(&("c".into(), 3)));
    }

    #[test]
    fn when_dict_has_bytes_keys_then_utf8_decoded() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Bytes(b"key_one".to_vec()),
            RencodeValue::Int(10),
        );
        map.insert(
            RencodeValue::Bytes(b"key_two".to_vec()),
            RencodeValue::Int(20),
        );
        let value = RencodeValue::Dict(map);

        let result: Vec<(String, i64)> = deserialize_dict_values(&value).expect("deserialize");

        assert_eq!(result.len(), 2);
        assert!(result.contains(&("key_one".into(), 10)));
        assert!(result.contains(&("key_two".into(), 20)));
    }

    #[test]
    fn when_dict_has_int_key_then_error() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Int(42), RencodeValue::Int(1));
        let value = RencodeValue::Dict(map);

        let result: Result<Vec<(String, i64)>, _> = deserialize_dict_values(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_dict_empty_then_empty_vec() {
        let value = RencodeValue::Dict(BTreeMap::new());

        let result: Vec<(String, i64)> = deserialize_dict_values(&value).expect("deserialize");

        assert!(result.is_empty());
    }

    #[test]
    fn when_value_type_mismatches_then_error() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("x".into()),
            RencodeValue::Str("not_an_int".into()),
        );
        let value = RencodeValue::Dict(map);

        let result: Result<Vec<(String, i64)>, _> = deserialize_dict_values(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_not_a_dict_then_error() {
        let value = RencodeValue::List(vec![RencodeValue::Int(1)]);

        let result: Result<Vec<(String, i64)>, _> = deserialize_dict_values(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_dict_has_bytes_key_invalid_utf8_then_error() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Bytes(vec![0xFF, 0xFE, 0xFF]),
            RencodeValue::Int(1),
        );
        let value = RencodeValue::Dict(map);

        let result: Result<Vec<(String, i64)>, _> = deserialize_dict_values(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_dict_has_struct_values_then_deserialized() {
        let mut map = BTreeMap::new();
        let mut inner_a = BTreeMap::new();
        inner_a.insert(
            RencodeValue::Str("name".into()),
            RencodeValue::Str("first".into()),
        );
        inner_a.insert(RencodeValue::Str("count".into()), RencodeValue::Int(10));
        map.insert(
            RencodeValue::Str("entry_a".into()),
            RencodeValue::Dict(inner_a),
        );

        let mut inner_b = BTreeMap::new();
        inner_b.insert(
            RencodeValue::Str("name".into()),
            RencodeValue::Str("second".into()),
        );
        inner_b.insert(RencodeValue::Str("count".into()), RencodeValue::Int(20));
        map.insert(
            RencodeValue::Str("entry_b".into()),
            RencodeValue::Dict(inner_b),
        );

        let value = RencodeValue::Dict(map);

        let result: Vec<(String, TestValue)> =
            deserialize_dict_values(&value).expect("deserialize");

        assert_eq!(result.len(), 2);
        assert!(result.contains(&(
            "entry_a".into(),
            TestValue {
                name: "first".into(),
                count: 10
            }
        )));
        assert!(result.contains(&(
            "entry_b".into(),
            TestValue {
                name: "second".into(),
                count: 20
            }
        )));
    }

    #[test]
    fn when_dict_has_bool_key_then_error() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Bool(true), RencodeValue::Int(1));
        let value = RencodeValue::Dict(map);

        let result: Result<Vec<(String, i64)>, _> = deserialize_dict_values(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_dict_has_float_key_then_error() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Float(1.5), RencodeValue::Int(1));
        let value = RencodeValue::Dict(map);

        let result: Result<Vec<(String, i64)>, _> = deserialize_dict_values(&value);

        assert!(result.is_err());
    }
}
