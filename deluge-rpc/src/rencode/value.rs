use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::Serialize;

use crate::rencode::decode::decode;
use crate::rencode::encode::encode;
use crate::rencode::error::RencodeError;
use crate::rencode::json::to_json;

#[derive(Debug, Clone)]
pub enum RencodeValue {
    None,
    Bool(bool),
    Int(i64),
    Str(String),
    Bytes(Vec<u8>),
    List(Vec<RencodeValue>),
    Dict(BTreeMap<RencodeValue, RencodeValue>),
    Float(f64),
}

impl RencodeValue {
    pub fn decode(data: &[u8]) -> Result<Self, RencodeError> {
        decode(data)
    }

    pub fn encode(&self) -> Vec<u8> {
        encode(self)
    }

    pub fn as_dict(&self) -> Result<&BTreeMap<RencodeValue, RencodeValue>, RencodeError> {
        match self {
            RencodeValue::Dict(map) => Ok(map),
            other => Err(RencodeError::WrongType {
                field: String::new(),
                expected: "dict",
                got: variant_name(other),
            }),
        }
    }

    pub fn get_str(&self, key: &str) -> Result<&str, RencodeError> {
        let field_key = RencodeValue::Str(String::from(key));
        let map = self.as_dict()?;
        match map.get(&field_key) {
            Some(RencodeValue::Str(s)) => Ok(s.as_str()),
            Some(other) => Err(RencodeError::WrongType {
                field: key.to_owned(),
                expected: "str",
                got: variant_name(other),
            }),
            None => Err(RencodeError::MissingField(key.to_owned())),
        }
    }

    pub fn get_int(&self, key: &str) -> Result<i64, RencodeError> {
        let field_key = RencodeValue::Str(String::from(key));
        let map = self.as_dict()?;
        match map.get(&field_key) {
            Some(RencodeValue::Int(i)) => Ok(*i),
            Some(RencodeValue::Float(f)) => Ok(*f as i64),
            Some(other) => Err(RencodeError::WrongType {
                field: key.to_owned(),
                expected: "int",
                got: variant_name(other),
            }),
            None => Err(RencodeError::MissingField(key.to_owned())),
        }
    }

    pub fn get_num(&self, key: &str) -> Result<f64, RencodeError> {
        let field_key = RencodeValue::Str(String::from(key));
        let map = self.as_dict()?;
        match map.get(&field_key) {
            Some(RencodeValue::Float(f)) => Ok(*f),
            Some(RencodeValue::Int(i)) => Ok(*i as f64),
            Some(other) => Err(RencodeError::WrongType {
                field: key.to_owned(),
                expected: "number",
                got: variant_name(other),
            }),
            None => Err(RencodeError::MissingField(key.to_owned())),
        }
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, RencodeError> {
        let field_key = RencodeValue::Str(String::from(key));
        let map = self.as_dict()?;
        match map.get(&field_key) {
            Some(RencodeValue::Bool(b)) => Ok(*b),
            Some(other) => Err(RencodeError::WrongType {
                field: key.to_owned(),
                expected: "bool",
                got: variant_name(other),
            }),
            None => Err(RencodeError::MissingField(key.to_owned())),
        }
    }
}

impl Serialize for RencodeValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let json = to_json(self);
        json.serialize(serializer)
    }
}

impl PartialOrd for RencodeValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RencodeValue {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for RencodeValue {}

impl Ord for RencodeValue {
    fn cmp(&self, other: &Self) -> Ordering {
        discriminant_tag(self)
            .cmp(&discriminant_tag(other))
            .then_with(|| compare_payload(self, other))
    }
}

fn discriminant_tag(v: &RencodeValue) -> u8 {
    match v {
        RencodeValue::None => 0,
        RencodeValue::Bool(_) => 1,
        RencodeValue::Int(_) => 2,
        RencodeValue::Str(_) => 3,
        RencodeValue::Bytes(_) => 4,
        RencodeValue::List(_) => 5,
        RencodeValue::Dict(_) => 6,
        RencodeValue::Float(_) => 7,
    }
}

fn compare_payload(a: &RencodeValue, b: &RencodeValue) -> Ordering {
    match (a, b) {
        (RencodeValue::Bool(x), RencodeValue::Bool(y)) => x.cmp(y),
        (RencodeValue::Int(x), RencodeValue::Int(y)) => x.cmp(y),
        (RencodeValue::Str(x), RencodeValue::Str(y)) => x.cmp(y),
        (RencodeValue::Bytes(x), RencodeValue::Bytes(y)) => x.cmp(y),
        (RencodeValue::List(x), RencodeValue::List(y)) => x.cmp(y),
        (RencodeValue::Dict(x), RencodeValue::Dict(y)) => x.cmp(y),
        (RencodeValue::Float(x), RencodeValue::Float(y)) => x.to_bits().cmp(&y.to_bits()),
        _ => Ordering::Equal,
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
