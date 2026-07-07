use crate::error::RencodeError;
use crate::value::RencodeValue;
use serde::ser::{self, Serialize};
use std::collections::BTreeMap;

/// Serialize a `Serialize` type into a `RencodeValue`.
pub fn to_rencode_value<T: Serialize + ?Sized>(value: &T) -> Result<RencodeValue, RencodeError> {
    value.serialize(Serializer)
}

struct Serializer;

struct SeqSerializer {
    items: Vec<RencodeValue>,
}

struct MapSerializer {
    map: BTreeMap<RencodeValue, RencodeValue>,
    next_key: Option<RencodeValue>,
}

struct StructSerializer {
    map: BTreeMap<RencodeValue, RencodeValue>,
}

impl ser::Serializer for Serializer {
    type Ok = RencodeValue;
    type Error = RencodeError;
    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = ser::Impossible<RencodeValue, RencodeError>;
    type SerializeMap = MapSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = ser::Impossible<RencodeValue, RencodeError>;

    fn serialize_bool(self, v: bool) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Int(i64::from(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Int(i64::from(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Int(i64::from(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Int(v))
    }

    fn serialize_u8(self, v: u8) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Int(i64::from(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Int(i64::from(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Int(i64::from(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<RencodeValue, RencodeError> {
        let i = i64::try_from(v).map_err(|_| {
            RencodeError::Message(format!("u64 value {v} out of i64 range for rencode"))
        })?;
        Ok(RencodeValue::Int(i))
    }

    fn serialize_f32(self, v: f32) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Float(f64::from(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Str(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Str(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Bytes(v.to_owned()))
    }

    fn serialize_none(self) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::None)
    }

    fn serialize_some<T: ?Sized + Serialize>(
        self,
        value: &T,
    ) -> Result<RencodeValue, RencodeError> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::None)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::None)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Str(variant.to_owned()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<RencodeValue, RencodeError> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<RencodeValue, RencodeError> {
        Err(ser::Error::custom(
            "newtype variants not supported in rencode",
        ))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<SeqSerializer, RencodeError> {
        Ok(SeqSerializer { items: Vec::new() })
    }

    fn serialize_tuple(self, _len: usize) -> Result<SeqSerializer, RencodeError> {
        Ok(SeqSerializer { items: Vec::new() })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<SeqSerializer, RencodeError> {
        Ok(SeqSerializer { items: Vec::new() })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<RencodeValue, RencodeError>, RencodeError> {
        Err(ser::Error::custom(
            "tuple variants not supported in rencode",
        ))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<MapSerializer, RencodeError> {
        Ok(MapSerializer {
            map: BTreeMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<StructSerializer, RencodeError> {
        Ok(StructSerializer {
            map: BTreeMap::new(),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<RencodeValue, RencodeError>, RencodeError> {
        Err(ser::Error::custom(
            "struct variants not supported in rencode",
        ))
    }
}

impl ser::SerializeSeq for SeqSerializer {
    type Ok = RencodeValue;
    type Error = RencodeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RencodeError> {
        self.items.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::List(self.items))
    }
}

impl ser::SerializeTuple for SeqSerializer {
    type Ok = RencodeValue;
    type Error = RencodeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RencodeError> {
        self.items.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::List(self.items))
    }
}

impl ser::SerializeTupleStruct for SeqSerializer {
    type Ok = RencodeValue;
    type Error = RencodeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RencodeError> {
        self.items.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::List(self.items))
    }
}

impl ser::SerializeMap for MapSerializer {
    type Ok = RencodeValue;
    type Error = RencodeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RencodeError> {
        self.next_key = Some(key.serialize(Serializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RencodeError> {
        let key = self.next_key.take().ok_or_else(|| {
            <RencodeError as ser::Error>::custom("serialize_value called before serialize_key")
        })?;
        self.map.insert(key, value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Dict(self.map))
    }
}

impl ser::SerializeStruct for StructSerializer {
    type Ok = RencodeValue;
    type Error = RencodeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RencodeError> {
        self.map.insert(
            RencodeValue::Str(key.to_owned()),
            value.serialize(Serializer)?,
        );
        Ok(())
    }

    fn end(self) -> Result<RencodeValue, RencodeError> {
        Ok(RencodeValue::Dict(self.map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize, PartialEq)]
    struct TestStruct {
        name: String,
        count: i64,
        active: bool,
    }

    #[test]
    fn when_struct_serialized_then_produces_dict() {
        let input = TestStruct {
            name: "test".into(),
            count: 42,
            active: true,
        };

        let result = to_rencode_value(&input).expect("serialize");

        match result {
            RencodeValue::Dict(map) => {
                assert_eq!(
                    map.get(&RencodeValue::Str("name".into())),
                    Some(&RencodeValue::Str("test".into()))
                );
                assert_eq!(
                    map.get(&RencodeValue::Str("count".into())),
                    Some(&RencodeValue::Int(42))
                );
                assert_eq!(
                    map.get(&RencodeValue::Str("active".into())),
                    Some(&RencodeValue::Bool(true))
                );
            }
            other => panic!("expected dict, got {other:?}"),
        }
    }

    #[test]
    fn when_vec_serialized_then_produces_list() {
        let input = vec![1i64, 2, 3];

        let result = to_rencode_value(&input).expect("serialize");

        match result {
            RencodeValue::List(items) => {
                assert_eq!(
                    items,
                    vec![
                        RencodeValue::Int(1),
                        RencodeValue::Int(2),
                        RencodeValue::Int(3)
                    ]
                );
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn when_option_some_serialized_then_produces_value() {
        let input = Some(42i64);

        let result = to_rencode_value(&input).expect("serialize");

        assert_eq!(result, RencodeValue::Int(42));
    }

    #[test]
    fn when_option_none_serialized_then_produces_none() {
        let input: Option<i64> = None;

        let result = to_rencode_value(&input).expect("serialize");

        assert_eq!(result, RencodeValue::None);
    }

    #[test]
    fn when_bool_serialized_then_produces_bool() {
        let result = to_rencode_value(&true).expect("serialize");
        assert_eq!(result, RencodeValue::Bool(true));
    }

    #[test]
    fn when_str_serialized_then_produces_str() {
        let result = to_rencode_value("hello").expect("serialize");
        assert_eq!(result, RencodeValue::Str("hello".into()));
    }

    #[test]
    #[expect(
        clippy::approx_constant,
        reason = "test value, not mathematical constant"
    )]
    fn when_f64_serialized_then_produces_float() {
        let result = to_rencode_value(&3.14f64).expect("serialize");
        assert_eq!(result, RencodeValue::Float(3.14));
    }

    #[test]
    fn when_unit_serialized_then_produces_none() {
        let result = to_rencode_value(&()).expect("serialize");
        assert_eq!(result, RencodeValue::None);
    }

    #[test]
    fn when_enum_unit_variant_serialized_then_produces_str() {
        #[derive(Debug, Serialize, PartialEq)]
        enum MyEnum {
            Foo,
            #[expect(dead_code, reason = "test variant exercises serialization code path")]
            Bar,
        }

        let result = to_rencode_value(&MyEnum::Foo).expect("serialize");
        assert_eq!(result, RencodeValue::Str("Foo".into()));
    }

    #[test]
    fn when_map_serialized_then_produces_dict() {
        let mut input = BTreeMap::new();
        input.insert("key1", 1i64);
        input.insert("key2", 2i64);

        let result = to_rencode_value(&input).expect("serialize");

        match result {
            RencodeValue::Dict(map) => {
                assert_eq!(map.len(), 2);
                assert_eq!(
                    map.get(&RencodeValue::Str("key1".into())),
                    Some(&RencodeValue::Int(1))
                );
                assert_eq!(
                    map.get(&RencodeValue::Str("key2".into())),
                    Some(&RencodeValue::Int(2))
                );
            }
            other => panic!("expected dict, got {other:?}"),
        }
    }
}
