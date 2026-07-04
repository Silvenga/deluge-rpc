use crate::rencode::decode::decode;
use crate::rencode::encode::encode;
use crate::rencode::error::RencodeError;
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// A rencode value covering all types needed for Deluge daemon RPC.
///
/// `Ord` is implemented so that [`RencodeValue`] can be used as a `BTreeMap`
/// key, giving deterministic dict encoding order. Floats are ordered by their
/// bit patterns (via `to_bits`), which is total and deterministic even though
/// it is not numerical order — acceptable since floats are never used as dict
/// keys in Deluge RPC.
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

pub fn discriminant_tag(v: &RencodeValue) -> u8 {
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

pub fn compare_payload(a: &RencodeValue, b: &RencodeValue) -> Ordering {
    match (a, b) {
        (RencodeValue::Bool(x), RencodeValue::Bool(y)) => x.cmp(y),
        (RencodeValue::Int(x), RencodeValue::Int(y)) => x.cmp(y),
        (RencodeValue::Str(x), RencodeValue::Str(y)) => x.cmp(y),
        (RencodeValue::Bytes(x), RencodeValue::Bytes(y)) => x.cmp(y),
        (RencodeValue::List(x), RencodeValue::List(y)) => x.cmp(y),
        (RencodeValue::Dict(x), RencodeValue::Dict(y)) => x.cmp(y),
        (RencodeValue::Float(x), RencodeValue::Float(y)) => x.to_bits().cmp(&y.to_bits()),
        // Different variants are already disambiguated by discriminant_tag.
        _ => Ordering::Equal,
    }
}
