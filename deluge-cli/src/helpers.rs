use base64::engine::general_purpose::STANDARD as Base64Engine;
use base64::Engine;
use deluge_rpc::RencodeValue;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

/// Converts plain JSON into an `RencodeValue`. This is the inverse of
/// [`rencode_to_plain_json`] and is used by the `call` command and
/// `core.set_config` to accept natural JSON on the CLI.
///
/// Mapping:
/// - JSON string → `RencodeValue::Str`
/// - JSON integer → `RencodeValue::Int`
/// - JSON float → `RencodeValue::Float`
/// - JSON bool → `RencodeValue::Bool`
/// - JSON null → `RencodeValue::None`
/// - JSON array → `RencodeValue::List` (recursive)
/// - JSON object → `RencodeValue::Dict` (recursive, keys as `RencodeValue::Str`)
pub fn rencode_from_json_value(json: &JsonValue) -> anyhow::Result<RencodeValue> {
    match json {
        JsonValue::Null => Ok(RencodeValue::None),
        JsonValue::Bool(b) => Ok(RencodeValue::Bool(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                if n.is_f64() && (n.as_f64().unwrap() - i as f64).abs() > f64::EPSILON {
                    Ok(RencodeValue::Float(n.as_f64().unwrap()))
                } else {
                    Ok(RencodeValue::Int(i))
                }
            } else if let Some(f) = n.as_f64() {
                Ok(RencodeValue::Float(f))
            } else {
                anyhow::bail!("unrepresentable JSON number: {n}")
            }
        }
        JsonValue::String(s) => Ok(RencodeValue::Str(s.clone())),
        JsonValue::Array(arr) => {
            let items: Result<Vec<RencodeValue>, _> =
                arr.iter().map(rencode_from_json_value).collect();
            Ok(RencodeValue::List(items?))
        }
        JsonValue::Object(obj) => {
            let mut map = BTreeMap::new();
            for (k, v) in obj {
                let key = RencodeValue::Str(k.clone());
                let val = rencode_from_json_value(v)?;
                map.insert(key, val);
            }
            Ok(RencodeValue::Dict(map))
        }
    }
}

/// Converts an `RencodeValue` into plain JSON. This is the inverse of
/// [`rencode_from_json_value`] and is used for CLI output where the tagged
/// JSON format (with `{"type": "int", "value": 42}`) is not desired.
///
/// Mapping:
/// - `RencodeValue::None` → JSON null
/// - `RencodeValue::Bool` → JSON bool
/// - `RencodeValue::Int` → JSON integer
/// - `RencodeValue::Float` → JSON float
/// - `RencodeValue::Str` → JSON string
/// - `RencodeValue::Bytes` → JSON string (base64:// prefixed)
/// - `RencodeValue::List` → JSON array (recursive)
/// - `RencodeValue::Dict` → JSON object (recursive, keys converted to strings)
pub fn rencode_to_plain_json(value: &RencodeValue) -> JsonValue {
    match value {
        RencodeValue::None => JsonValue::Null,
        RencodeValue::Bool(b) => JsonValue::Bool(*b),
        RencodeValue::Int(i) => JsonValue::Number((*i).into()),
        RencodeValue::Float(f) => JsonValue::Number(
            serde_json::Number::from_f64(*f)
                .unwrap_or_else(|| serde_json::Number::from_f64(0.0).unwrap()),
        ),
        RencodeValue::Str(s) => JsonValue::String(s.clone()),
        RencodeValue::Bytes(b) => {
            let encoded = Base64Engine.encode(b);
            JsonValue::String(format!("base64://{encoded}"))
        }
        RencodeValue::List(items) => {
            JsonValue::Array(items.iter().map(rencode_to_plain_json).collect())
        }
        RencodeValue::Dict(dict) => {
            let mut map = serde_json::Map::new();
            for (k, v) in dict {
                let key_str = match rencode_to_plain_json(k) {
                    JsonValue::String(s) => s,
                    other => other.to_string(),
                };
                map.insert(key_str, rencode_to_plain_json(v));
            }
            JsonValue::Object(map)
        }
    }
}
