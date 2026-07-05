use crate::rencode::error::RencodeError;
use crate::rencode::value::RencodeValue;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as Base64Engine;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

pub fn to_json(value: &RencodeValue) -> JsonValue {
    match value {
        RencodeValue::None => {
            let mut map = serde_json::Map::new();
            map.insert("type".to_owned(), JsonValue::String("none".to_owned()));
            JsonValue::Object(map)
        }
        RencodeValue::Bool(b) => {
            let mut map = serde_json::Map::new();
            map.insert("type".to_owned(), JsonValue::String("bool".to_owned()));
            map.insert("value".to_owned(), JsonValue::Bool(*b));
            JsonValue::Object(map)
        }
        RencodeValue::Int(i) => {
            let mut map = serde_json::Map::new();
            map.insert("type".to_owned(), JsonValue::String("int".to_owned()));
            map.insert("value".to_owned(), JsonValue::Number((*i).into()));
            JsonValue::Object(map)
        }
        RencodeValue::Float(f) => {
            let mut map = serde_json::Map::new();
            map.insert("type".to_owned(), JsonValue::String("float".to_owned()));
            map.insert(
                "value".to_owned(),
                JsonValue::Number(
                    serde_json::Number::from_f64(*f)
                        .unwrap_or_else(|| serde_json::Number::from_f64(0.0).unwrap()),
                ),
            );
            JsonValue::Object(map)
        }
        RencodeValue::Str(s) => {
            let mut map = serde_json::Map::new();
            map.insert("type".to_owned(), JsonValue::String("str".to_owned()));
            map.insert("value".to_owned(), JsonValue::String(s.clone()));
            JsonValue::Object(map)
        }
        RencodeValue::Bytes(b) => {
            let mut map = serde_json::Map::new();
            map.insert("type".to_owned(), JsonValue::String("bytes".to_owned()));
            let encoded = Base64Engine.encode(b);
            map.insert(
                "value".to_owned(),
                JsonValue::String(format!("base64://{encoded}")),
            );
            JsonValue::Object(map)
        }
        RencodeValue::List(items) => {
            let mut map = serde_json::Map::new();
            map.insert("type".to_owned(), JsonValue::String("list".to_owned()));
            let values: Vec<JsonValue> = items.iter().map(to_json).collect();
            map.insert("value".to_owned(), JsonValue::Array(values));
            JsonValue::Object(map)
        }
        RencodeValue::Dict(dict) => {
            let mut map = serde_json::Map::new();
            map.insert("type".to_owned(), JsonValue::String("dict".to_owned()));
            let pairs: Vec<JsonValue> = dict
                .iter()
                .map(|(k, v)| {
                    let mut pair = serde_json::Map::new();
                    pair.insert("key".to_owned(), to_json(k));
                    pair.insert("value".to_owned(), to_json(v));
                    JsonValue::Object(pair)
                })
                .collect();
            map.insert("value".to_owned(), JsonValue::Array(pairs));
            JsonValue::Object(map)
        }
    }
}

pub fn from_json(json: &JsonValue) -> Result<RencodeValue, RencodeError> {
    let obj = json
        .as_object()
        .ok_or_else(|| RencodeError::InvalidTaggedJson("expected JSON object".to_owned()))?;

    let type_str = obj.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
        RencodeError::InvalidTaggedJson("missing or invalid 'type' field".to_owned())
    })?;

    match type_str {
        "none" => Ok(RencodeValue::None),
        "bool" => {
            let val = obj.get("value").and_then(|v| v.as_bool()).ok_or_else(|| {
                RencodeError::InvalidTaggedJson("missing or invalid 'value' for bool".to_owned())
            })?;
            Ok(RencodeValue::Bool(val))
        }
        "int" => {
            let val = obj.get("value").and_then(|v| v.as_i64()).ok_or_else(|| {
                RencodeError::InvalidTaggedJson("missing or invalid 'value' for int".to_owned())
            })?;
            Ok(RencodeValue::Int(val))
        }
        "float" => {
            let val = obj.get("value").and_then(|v| v.as_f64()).ok_or_else(|| {
                RencodeError::InvalidTaggedJson("missing or invalid 'value' for float".to_owned())
            })?;
            Ok(RencodeValue::Float(val))
        }
        "str" => {
            let val = obj.get("value").and_then(|v| v.as_str()).ok_or_else(|| {
                RencodeError::InvalidTaggedJson("missing or invalid 'value' for str".to_owned())
            })?;
            Ok(RencodeValue::Str(val.to_owned()))
        }
        "bytes" => {
            let val = obj.get("value").and_then(|v| v.as_str()).ok_or_else(|| {
                RencodeError::InvalidTaggedJson("missing or invalid 'value' for bytes".to_owned())
            })?;
            let encoded = val.strip_prefix("base64://").ok_or_else(|| {
                RencodeError::InvalidTaggedJson("bytes value missing 'base64://' prefix".to_owned())
            })?;
            let decoded = Base64Engine.decode(encoded).map_err(|e| {
                RencodeError::InvalidTaggedJson(format!("base64 decode error: {e}"))
            })?;
            Ok(RencodeValue::Bytes(decoded))
        }
        "list" => {
            let arr = obj.get("value").and_then(|v| v.as_array()).ok_or_else(|| {
                RencodeError::InvalidTaggedJson("missing or invalid 'value' for list".to_owned())
            })?;
            let items: Result<Vec<RencodeValue>, RencodeError> =
                arr.iter().map(from_json).collect();
            Ok(RencodeValue::List(items?))
        }
        "dict" => {
            let arr = obj.get("value").and_then(|v| v.as_array()).ok_or_else(|| {
                RencodeError::InvalidTaggedJson("missing or invalid 'value' for dict".to_owned())
            })?;
            let mut map = BTreeMap::new();
            for pair_json in arr {
                let pair_obj = pair_json.as_object().ok_or_else(|| {
                    RencodeError::InvalidTaggedJson("dict pair is not an object".to_owned())
                })?;
                let key_json = pair_obj.get("key").ok_or_else(|| {
                    RencodeError::InvalidTaggedJson("dict pair missing 'key'".to_owned())
                })?;
                let value_json = pair_obj.get("value").ok_or_else(|| {
                    RencodeError::InvalidTaggedJson("dict pair missing 'value'".to_owned())
                })?;
                let key = from_json(key_json)?;
                let value = from_json(value_json)?;
                map.insert(key, value);
            }
            Ok(RencodeValue::Dict(map))
        }
        other => Err(RencodeError::InvalidTaggedJson(format!(
            "unknown type tag: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    #[expect(clippy::approx_constant, reason = "test values for float round-trip")]
    fn when_roundtrip_all_variants_then_equal() {
        let values = vec![
            RencodeValue::None,
            RencodeValue::Bool(true),
            RencodeValue::Bool(false),
            RencodeValue::Int(0),
            RencodeValue::Int(42),
            RencodeValue::Int(-1),
            RencodeValue::Int(i64::MAX),
            RencodeValue::Int(i64::MIN),
            RencodeValue::Float(0.0),
            RencodeValue::Float(1.5),
            RencodeValue::Float(-3.14),
            RencodeValue::Str(String::new()),
            RencodeValue::Str("hello".to_owned()),
            RencodeValue::Bytes(vec![]),
            RencodeValue::Bytes(vec![0xFF, 0x00, 0xAB]),
            RencodeValue::List(vec![]),
            RencodeValue::List(vec![
                RencodeValue::Int(1),
                RencodeValue::Str("two".to_owned()),
            ]),
            RencodeValue::Dict(BTreeMap::new()),
        ];

        for original in values {
            let json = to_json(&original);
            let roundtripped = from_json(&json).expect("roundtrip should succeed");
            assert_eq!(original, roundtripped, "roundtrip failed for {original:?}");
        }
    }

    #[test]
    fn when_bytes_then_base64_encoded() {
        let value = RencodeValue::Bytes(vec![0xFF, 0x00]);
        let json = to_json(&value);

        let obj = json.as_object().expect("should be object");
        assert_eq!(obj["type"], json!("bytes"));
        let val_str = obj["value"].as_str().expect("value should be string");
        assert!(
            val_str.starts_with("base64://"),
            "bytes value should have base64:// prefix, got: {val_str}"
        );

        let roundtripped = from_json(&json).expect("roundtrip should succeed");
        assert_eq!(value, roundtripped);
    }

    #[test]
    fn when_dict_with_int_keys_then_pairs_array() {
        let mut dict = BTreeMap::new();
        dict.insert(RencodeValue::Int(1), RencodeValue::Str("one".to_owned()));
        dict.insert(RencodeValue::Int(2), RencodeValue::Str("two".to_owned()));
        let value = RencodeValue::Dict(dict);

        let json = to_json(&value);
        let obj = json.as_object().expect("should be object");
        assert_eq!(obj["type"], json!("dict"));

        let pairs = obj["value"].as_array().expect("value should be array");
        assert_eq!(pairs.len(), 2);

        for pair in pairs {
            let pair_obj = pair.as_object().expect("pair should be object");
            assert!(pair_obj.contains_key("key"));
            assert!(pair_obj.contains_key("value"));
        }

        let roundtripped = from_json(&json).expect("roundtrip should succeed");
        assert_eq!(value, roundtripped);
    }

    #[test]
    fn when_dict_with_string_keys_then_pairs_array() {
        let mut dict = BTreeMap::new();
        dict.insert(RencodeValue::Str("a".to_owned()), RencodeValue::Int(1));
        dict.insert(RencodeValue::Str("b".to_owned()), RencodeValue::Int(2));
        let value = RencodeValue::Dict(dict);

        let json = to_json(&value);
        let obj = json.as_object().expect("should be object");
        assert_eq!(obj["type"], json!("dict"));

        let pairs = obj["value"].as_array().expect("value should be array");
        assert_eq!(pairs.len(), 2);

        let roundtripped = from_json(&json).expect("roundtrip should succeed");
        assert_eq!(value, roundtripped);
    }

    #[test]
    fn when_none_then_type_tag() {
        let value = RencodeValue::None;
        let json = to_json(&value);

        let obj = json.as_object().expect("should be object");
        assert_eq!(obj["type"], json!("none"));
        assert!(
            !obj.contains_key("value"),
            "None should not have a value field"
        );

        let roundtripped = from_json(&json).expect("roundtrip should succeed");
        assert_eq!(value, roundtripped);
    }

    #[test]
    fn when_float_then_preserved() {
        let value = RencodeValue::Float(1.5);
        let json = to_json(&value);

        let obj = json.as_object().expect("should be object");
        assert_eq!(obj["type"], json!("float"));
        assert_eq!(obj["value"], json!(1.5));

        let roundtripped = from_json(&json).expect("roundtrip should succeed");
        assert_eq!(value, roundtripped);
    }

    #[test]
    fn when_nested_structures_then_recursive_roundtrip() {
        let mut inner_dict = BTreeMap::new();
        inner_dict.insert(RencodeValue::Str("key".to_owned()), RencodeValue::Int(42));

        let value = RencodeValue::List(vec![
            RencodeValue::Int(1),
            RencodeValue::Dict(inner_dict),
            RencodeValue::List(vec![
                RencodeValue::Str("nested".to_owned()),
                RencodeValue::Bool(true),
            ]),
        ]);

        let json = to_json(&value);
        let roundtripped = from_json(&json).expect("roundtrip should succeed");
        assert_eq!(value, roundtripped);
    }

    #[test]
    fn when_invalid_tagged_json_then_error() {
        let json = json!({"type": "unknown"});
        let result = from_json(&json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown type tag"), "got: {msg}");
    }

    #[test]
    fn when_missing_type_field_then_error() {
        let json = json!({"value": 42});
        let result = from_json(&json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing or invalid 'type'"), "got: {msg}");
    }

    #[test]
    fn when_not_an_object_then_error() {
        let json = json!(42);
        let result = from_json(&json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("expected JSON object"), "got: {msg}");
    }

    #[test]
    fn when_bytes_missing_prefix_then_error() {
        let json = json!({"type": "bytes", "value": "not-base64"});
        let result = from_json(&json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("base64://"), "got: {msg}");
    }

    #[test]
    fn when_dict_preserves_key_order() {
        let mut dict = BTreeMap::new();
        dict.insert(RencodeValue::Int(3), RencodeValue::Str("c".to_owned()));
        dict.insert(RencodeValue::Int(1), RencodeValue::Str("a".to_owned()));
        dict.insert(RencodeValue::Int(2), RencodeValue::Str("b".to_owned()));
        let value = RencodeValue::Dict(dict);

        let json = to_json(&value);
        let roundtripped = from_json(&json).expect("roundtrip should succeed");
        assert_eq!(value, roundtripped);
    }
}
