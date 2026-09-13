use deluge_rpc_rencode::RencodeValue;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};

/// Serializes a `RencodeValue` as its plain rencode representation, bypassing the tagged-JSON
/// serde impl so values reach the daemon in wire format.
pub(crate) struct RawRencode<'a>(pub(crate) &'a RencodeValue);

impl Serialize for RawRencode<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            RencodeValue::None => serializer.serialize_unit(),
            RencodeValue::Bool(v) => serializer.serialize_bool(*v),
            RencodeValue::Int(v) => serializer.serialize_i64(*v),
            RencodeValue::Str(v) => serializer.serialize_str(v),
            RencodeValue::Bytes(v) => serializer.serialize_bytes(v),
            RencodeValue::Float(v) => serializer.serialize_f64(*v),
            RencodeValue::List(items) => {
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(&RawRencode(item))?;
                }
                seq.end()
            }
            RencodeValue::Dict(map) => {
                let mut map_ser = serializer.serialize_map(Some(map.len()))?;
                for (key, value) in map {
                    map_ser.serialize_entry(&RawRencode(key), &RawRencode(value))?;
                }
                map_ser.end()
            }
        }
    }
}

pub(crate) struct RawList<'a>(pub(crate) &'a [RencodeValue]);

impl Serialize for RawList<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for item in self.0 {
            seq.serialize_element(&RawRencode(item))?;
        }
        seq.end()
    }
}
