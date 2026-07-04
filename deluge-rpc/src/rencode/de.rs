use crate::rencode::de_helpers::{
    EnumDeserializer, EnumRefDeserializer, MapDeserializer, MapRefDeserializer, SeqDeserializer,
    SeqRefDeserializer,
};
use crate::rencode::error::RencodeError;
use crate::rencode::value::RencodeValue;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Unexpected, Visitor};
use serde::forward_to_deserialize_any;
use std::collections::BTreeMap;
use std::fmt;
use std::str::from_utf8;

impl<'de> Deserialize<'de> for RencodeValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = RencodeValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid rencode value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<RencodeValue, E> {
                Ok(RencodeValue::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<RencodeValue, E> {
                Ok(RencodeValue::Int(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<RencodeValue, E> {
                Ok(RencodeValue::Float(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<RencodeValue, E>
            where
                E: de::Error,
            {
                Ok(RencodeValue::Str(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<RencodeValue, E> {
                Ok(RencodeValue::Str(value))
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<RencodeValue, E>
            where
                E: de::Error,
            {
                Ok(RencodeValue::Bytes(value.to_owned()))
            }

            fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<RencodeValue, E> {
                Ok(RencodeValue::Bytes(value))
            }

            fn visit_none<E>(self) -> Result<RencodeValue, E> {
                Ok(RencodeValue::None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<RencodeValue, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_unit<E>(self) -> Result<RencodeValue, E> {
                Ok(RencodeValue::None)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<RencodeValue, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }
                Ok(RencodeValue::List(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<RencodeValue, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut map = BTreeMap::new();
                while let Some((key, value)) = visitor.next_entry()? {
                    map.insert(key, value);
                }
                Ok(RencodeValue::Dict(map))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

pub(super) fn unexpected_from_value(value: &RencodeValue) -> Unexpected<'_> {
    match value {
        RencodeValue::None => Unexpected::Unit,
        RencodeValue::Bool(b) => Unexpected::Bool(*b),
        RencodeValue::Int(i) => Unexpected::Signed(*i),
        RencodeValue::Str(s) => Unexpected::Str(s),
        RencodeValue::Bytes(b) => Unexpected::Bytes(b),
        RencodeValue::List(_) => Unexpected::Seq,
        RencodeValue::Dict(_) => Unexpected::Map,
        RencodeValue::Float(f) => Unexpected::Float(*f),
    }
}

pub(super) fn key_to_string(key: &RencodeValue) -> Result<String, RencodeError> {
    match key {
        RencodeValue::Str(s) => Ok(s.clone()),
        RencodeValue::Bytes(b) => from_utf8(b)
            .map(|s| s.to_owned())
            .map_err(|_| RencodeError::Message("dict key bytes are not valid UTF-8".into())),
        RencodeValue::Int(i) => Ok(i.to_string()),
        RencodeValue::Bool(b) => Ok(b.to_string()),
        RencodeValue::Float(f) => Ok(f.to_string()),
        RencodeValue::List(_) => Err(RencodeError::Message("list cannot be a dict key".into())),
        RencodeValue::Dict(_) => Err(RencodeError::Message("dict cannot be a dict key".into())),
        RencodeValue::None => Err(RencodeError::Message("none cannot be a dict key".into())),
    }
}

pub(super) fn visit_list<'de, V>(
    list: Vec<RencodeValue>,
    visitor: V,
) -> Result<V::Value, RencodeError>
where
    V: Visitor<'de>,
{
    let len = list.len();
    let mut deserializer = SeqDeserializer {
        iter: list.into_iter(),
    };
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(de::Error::invalid_length(len, &"fewer elements in list"))
    }
}

pub(super) fn visit_list_ref<'de, V>(
    list: &'de [RencodeValue],
    visitor: V,
) -> Result<V::Value, RencodeError>
where
    V: Visitor<'de>,
{
    let len = list.len();
    let mut deserializer = SeqRefDeserializer { iter: list.iter() };
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(de::Error::invalid_length(len, &"fewer elements in list"))
    }
}

pub(super) fn visit_dict<'de, V>(
    dict: BTreeMap<RencodeValue, RencodeValue>,
    visitor: V,
) -> Result<V::Value, RencodeError>
where
    V: Visitor<'de>,
{
    let len = dict.len();
    let mut deserializer = MapDeserializer {
        iter: dict.into_iter(),
        value: None,
    };
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(de::Error::invalid_length(len, &"fewer elements in map"))
    }
}

pub(super) fn visit_dict_ref<'de, V>(
    dict: &'de BTreeMap<RencodeValue, RencodeValue>,
    visitor: V,
) -> Result<V::Value, RencodeError>
where
    V: Visitor<'de>,
{
    let len = dict.len();
    let mut deserializer = MapRefDeserializer {
        iter: dict.iter(),
        value: None,
    };
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(de::Error::invalid_length(len, &"fewer elements in map"))
    }
}

impl<'de> Deserializer<'de> for RencodeValue {
    type Error = RencodeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::None => visitor.visit_unit(),
            RencodeValue::Bool(v) => visitor.visit_bool(v),
            RencodeValue::Int(v) => visitor.visit_i64(v),
            RencodeValue::Str(v) => visitor.visit_string(v),
            RencodeValue::Bytes(v) => visitor.visit_byte_buf(v),
            RencodeValue::List(v) => visit_list(v, visitor),
            RencodeValue::Dict(v) => visit_dict(v, visitor),
            RencodeValue::Float(v) => visitor.visit_f64(v),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Bool(v) => visitor.visit_bool(v),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(&self),
                &visitor,
            )),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Int(v) => visitor.visit_i64(v),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(&self),
                &visitor,
            )),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Int(v) => {
                let u = v as u64;
                visitor.visit_u64(u)
            }
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(&self),
                &visitor,
            )),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Float(v) => visitor.visit_f64(v),
            RencodeValue::Int(v) => visitor.visit_f64(v as f64),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(&self),
                &visitor,
            )),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Str(v) => visitor.visit_string(v),
            RencodeValue::Bytes(v) => {
                let s = from_utf8(&v)
                    .map_err(|_| de::Error::invalid_type(Unexpected::Bytes(&v), &visitor))?;
                visitor.visit_string(s.to_owned())
            }
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(&self),
                &visitor,
            )),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Bytes(v) => visitor.visit_byte_buf(v),
            RencodeValue::Str(v) => visitor.visit_bytes(v.as_bytes()),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(&self),
                &visitor,
            )),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::None => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::None => visitor.visit_unit(),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(&self),
                &visitor,
            )),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::List(v) => visit_list(v, visitor),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(&self),
                &visitor,
            )),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Dict(v) => visit_dict(v, visitor),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(&self),
                &visitor,
            )),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Dict(v) => visit_dict(v, visitor),
            RencodeValue::List(v) => visit_list(v, visitor),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(&self),
                &visitor,
            )),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Dict(v) => {
                if v.len() != 1 {
                    return Err(de::Error::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                let (key, value) = v.into_iter().next().unwrap();
                let variant = key_to_string(&key)?;
                visitor.visit_enum(EnumDeserializer {
                    variant,
                    value: Some(value),
                })
            }
            RencodeValue::Str(v) => visitor.visit_enum(EnumDeserializer {
                variant: v,
                value: None,
            }),
            other => Err(de::Error::invalid_type(
                unexpected_from_value(&other),
                &"string or map",
            )),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i128 u8 u16 u32 u128 f32
    }
}

impl<'de> Deserializer<'de> for &'de RencodeValue {
    type Error = RencodeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::None => visitor.visit_unit(),
            RencodeValue::Bool(v) => visitor.visit_bool(*v),
            RencodeValue::Int(v) => visitor.visit_i64(*v),
            RencodeValue::Str(v) => visitor.visit_borrowed_str(v),
            RencodeValue::Bytes(v) => visitor.visit_borrowed_bytes(v),
            RencodeValue::List(v) => visit_list_ref(v, visitor),
            RencodeValue::Dict(v) => visit_dict_ref(v, visitor),
            RencodeValue::Float(v) => visitor.visit_f64(*v),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Bool(v) => visitor.visit_bool(*v),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(self),
                &visitor,
            )),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Int(v) => visitor.visit_i64(*v),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(self),
                &visitor,
            )),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Int(v) => {
                let u = *v as u64;
                visitor.visit_u64(u)
            }
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(self),
                &visitor,
            )),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Float(v) => visitor.visit_f64(*v),
            RencodeValue::Int(v) => visitor.visit_f64(*v as f64),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(self),
                &visitor,
            )),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Str(v) => visitor.visit_borrowed_str(v),
            RencodeValue::Bytes(v) => {
                let s = from_utf8(v)
                    .map_err(|_| de::Error::invalid_type(Unexpected::Bytes(v), &visitor))?;
                visitor.visit_borrowed_str(s)
            }
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(self),
                &visitor,
            )),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Bytes(v) => visitor.visit_borrowed_bytes(v),
            RencodeValue::Str(v) => visitor.visit_borrowed_str(v),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(self),
                &visitor,
            )),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::None => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::None => visitor.visit_unit(),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(self),
                &visitor,
            )),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::List(v) => visit_list_ref(v, visitor),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(self),
                &visitor,
            )),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Dict(v) => visit_dict_ref(v, visitor),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(self),
                &visitor,
            )),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Dict(v) => visit_dict_ref(v, visitor),
            RencodeValue::List(v) => visit_list_ref(v, visitor),
            _ => Err(de::Error::invalid_type(
                unexpected_from_value(self),
                &visitor,
            )),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            RencodeValue::Dict(v) => {
                if v.len() != 1 {
                    return Err(de::Error::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                let (key, value) = v.iter().next().unwrap();
                let variant = key_to_string(key)?;
                visitor.visit_enum(EnumRefDeserializer {
                    variant,
                    value: Some(value),
                })
            }
            RencodeValue::Str(v) => visitor.visit_enum(EnumRefDeserializer {
                variant: v.clone(),
                value: None,
            }),
            other => Err(de::Error::invalid_type(
                unexpected_from_value(other),
                &"string or map",
            )),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i128 u8 u16 u32 u128 f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        name: String,
        count: i64,
        active: bool,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Outer {
        a: bool,
        #[serde(flatten)]
        inner: Inner,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Inner {
        b: i64,
        c: String,
    }

    fn make_test_dict() -> RencodeValue {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("name".into()),
            RencodeValue::Str("test".into()),
        );
        map.insert(RencodeValue::Str("count".into()), RencodeValue::Int(42));
        map.insert(RencodeValue::Str("active".into()), RencodeValue::Bool(true));
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_deserialize_struct_from_dict_then_fields_populate() {
        let value = make_test_dict();

        let result: TestStruct = TestStruct::deserialize(&value).expect("deserialize");

        assert_eq!(
            result,
            TestStruct {
                name: "test".into(),
                count: 42,
                active: true,
            }
        );
    }

    #[test]
    fn when_deserialize_vec_from_list_then_items_populate() {
        let value = RencodeValue::List(vec![
            RencodeValue::Int(1),
            RencodeValue::Int(2),
            RencodeValue::Int(3),
        ]);

        let result: Vec<i64> = Vec::deserialize(&value).expect("deserialize");

        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn when_deserialize_option_from_none_then_none() {
        let value = RencodeValue::None;

        let result: Option<i64> = Option::deserialize(&value).expect("deserialize");

        assert_eq!(result, None);
    }

    #[test]
    fn when_deserialize_option_from_value_then_some() {
        let value = RencodeValue::Int(42);

        let result: Option<i64> = Option::deserialize(&value).expect("deserialize");

        assert_eq!(result, Some(42));
    }

    #[test]
    fn when_dict_has_int_key_then_string_key_deserializer_coerces() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Int(1), RencodeValue::Int(100));
        map.insert(RencodeValue::Int(2), RencodeValue::Int(200));
        let value = RencodeValue::Dict(map);

        let result: HashMap<String, i64> = HashMap::deserialize(&value).expect("deserialize");

        let mut expected = HashMap::new();
        expected.insert("1".into(), 100);
        expected.insert("2".into(), 200);
        assert_eq!(result, expected);
    }

    #[test]
    fn when_dict_has_bytes_key_then_string_key_deserializer_decodes_utf8() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Bytes(b"hello".to_vec()),
            RencodeValue::Int(42),
        );
        let value = RencodeValue::Dict(map);

        let result: HashMap<String, i64> = HashMap::deserialize(&value).expect("deserialize");

        let mut expected = HashMap::new();
        expected.insert("hello".into(), 42);
        assert_eq!(result, expected);
    }

    #[test]
    fn when_deserialize_flatten_then_inner_fields_merged() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("a".into()), RencodeValue::Bool(true));
        map.insert(RencodeValue::Str("b".into()), RencodeValue::Int(10));
        map.insert(
            RencodeValue::Str("c".into()),
            RencodeValue::Str("inner".into()),
        );
        let value = RencodeValue::Dict(map);

        let result: Outer = Outer::deserialize(&value).expect("deserialize");

        assert_eq!(
            result,
            Outer {
                a: true,
                inner: Inner {
                    b: 10,
                    c: "inner".into(),
                },
            }
        );
    }

    #[test]
    fn when_deserialize_wrong_type_then_error() {
        let value = RencodeValue::Int(42);

        let result: Result<String, _> = String::deserialize(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_deserialize_owned_value_then_works() {
        let value = RencodeValue::Int(42);

        let result: i64 = i64::deserialize(value).expect("deserialize");

        assert_eq!(result, 42);
    }

    #[test]
    fn when_deserialize_float_from_int_then_converts() {
        let value = RencodeValue::Int(42);

        let result: f64 = f64::deserialize(&value).expect("deserialize");

        assert!((result - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn when_deserialize_u64_from_int_then_converts() {
        let value = RencodeValue::Int(42);

        let result: u64 = u64::deserialize(&value).expect("deserialize");

        assert_eq!(result, 42);
    }

    #[test]
    fn when_deserialize_str_from_bytes_then_works() {
        let value = RencodeValue::Bytes(b"hello".to_vec());

        let result: String = String::deserialize(&value).expect("deserialize");

        assert_eq!(result, "hello");
    }

    #[test]
    fn when_deserialize_bool_from_int_then_error() {
        let value = RencodeValue::Int(1);

        let result: Result<bool, _> = bool::deserialize(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_deserialize_enum_unit_variant_from_str_then_works() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum MyEnum {
            Foo,
            Bar,
        }

        let value = RencodeValue::Str("Foo".into());

        let result: MyEnum = MyEnum::deserialize(&value).expect("deserialize");

        assert_eq!(result, MyEnum::Foo);
    }

    #[test]
    fn when_deserialize_enum_newtype_variant_from_dict_then_works() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum MyEnum {
            Foo(i64),
        }

        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("Foo".into()), RencodeValue::Int(42));
        let value = RencodeValue::Dict(map);

        let result: MyEnum = MyEnum::deserialize(&value).expect("deserialize");

        assert_eq!(result, MyEnum::Foo(42));
    }

    #[test]
    fn when_deserialize_empty_list_then_works() {
        let value = RencodeValue::List(vec![]);

        let result: Vec<i64> = Vec::deserialize(&value).expect("deserialize");

        assert_eq!(result, Vec::<i64>::new());
    }

    #[test]
    fn when_deserialize_empty_dict_then_works() {
        let value = RencodeValue::Dict(BTreeMap::new());

        let result: HashMap<String, i64> = HashMap::deserialize(&value).expect("deserialize");

        assert!(result.is_empty());
    }

    #[test]
    fn when_deserialize_tuple_from_list_then_works() {
        let value = RencodeValue::List(vec![
            RencodeValue::Int(1),
            RencodeValue::Str("two".into()),
            RencodeValue::Bool(true),
        ]);

        let result: (i64, String, bool) =
            <(i64, String, bool)>::deserialize(&value).expect("deserialize");

        assert_eq!(result, (1, "two".into(), true));
    }

    #[test]
    fn when_deserialize_borrowed_str_then_zero_copy() {
        let value = RencodeValue::Str("hello".into());

        let result: &str = <&str>::deserialize(&value).expect("deserialize");

        assert_eq!(result, "hello");
    }

    #[test]
    fn when_deserialize_borrowed_bytes_then_zero_copy() {
        let value = RencodeValue::Bytes(b"hello".to_vec());

        let result: &[u8] = <&[u8]>::deserialize(&value).expect("deserialize");

        assert_eq!(result, b"hello");
    }

    #[test]
    fn when_deserialize_ignored_any_then_consumes_value() {
        use serde::de::IgnoredAny;

        let value = RencodeValue::List(vec![RencodeValue::Int(1), RencodeValue::Int(2)]);

        let result = IgnoredAny::deserialize(&value);

        assert!(result.is_ok());
    }

    #[test]
    fn when_deserialize_newtype_struct_then_works() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Wrapper(i64);

        let value = RencodeValue::Int(42);

        let result: Wrapper = Wrapper::deserialize(&value).expect("deserialize");

        assert_eq!(result, Wrapper(42));
    }

    #[test]
    fn when_deserialize_unit_then_works() {
        let value = RencodeValue::None;

        let result: () = <()>::deserialize(&value).expect("deserialize");

        assert_eq!(result, ());
    }

    #[test]
    fn when_deserialize_unit_from_non_none_then_error() {
        let value = RencodeValue::Int(42);

        let result: Result<(), _> = <()>::deserialize(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_deserialize_bool_key_then_string_key_deserializer_coerces() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Bool(true), RencodeValue::Int(1));
        map.insert(RencodeValue::Bool(false), RencodeValue::Int(0));
        let value = RencodeValue::Dict(map);

        let result: HashMap<String, i64> = HashMap::deserialize(&value).expect("deserialize");

        let mut expected = HashMap::new();
        expected.insert("true".into(), 1);
        expected.insert("false".into(), 0);
        assert_eq!(result, expected);
    }

    #[test]
    fn when_deserialize_float_key_then_string_key_deserializer_coerces() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Float(1.5), RencodeValue::Int(42));
        let value = RencodeValue::Dict(map);

        let result: HashMap<String, i64> = HashMap::deserialize(&value).expect("deserialize");

        let mut expected = HashMap::new();
        expected.insert("1.5".into(), 42);
        assert_eq!(result, expected);
    }

    #[test]
    fn when_deserialize_list_key_then_error() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::List(vec![RencodeValue::Int(1)]),
            RencodeValue::Int(42),
        );
        let value = RencodeValue::Dict(map);

        let result: Result<HashMap<String, i64>, _> = HashMap::deserialize(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_deserialize_none_key_then_error() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::None, RencodeValue::Int(42));
        let value = RencodeValue::Dict(map);

        let result: Result<HashMap<String, i64>, _> = HashMap::deserialize(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_deserialize_non_utf8_bytes_key_then_error() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Bytes(vec![0xFF, 0xFE]), RencodeValue::Int(42));
        let value = RencodeValue::Dict(map);

        let result: Result<HashMap<String, i64>, _> = HashMap::deserialize(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_deserialize_enum_from_multi_key_dict_then_error() {
        #[derive(Debug, Deserialize)]
        enum MyEnum {
            Foo,
        }

        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("Foo".into()), RencodeValue::Int(1));
        map.insert(RencodeValue::Str("Bar".into()), RencodeValue::Int(2));
        let value = RencodeValue::Dict(map);

        let result: Result<MyEnum, _> = MyEnum::deserialize(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_deserialize_enum_from_non_str_non_dict_then_error() {
        #[derive(Debug, Deserialize)]
        enum MyEnum {
            Foo,
        }

        let value = RencodeValue::Int(42);

        let result: Result<MyEnum, _> = MyEnum::deserialize(&value);

        assert!(result.is_err());
    }

    #[test]
    fn when_deserialize_enum_struct_variant_then_works() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum MyEnum {
            Foo { x: i64, y: String },
        }

        let mut inner = BTreeMap::new();
        inner.insert(RencodeValue::Str("x".into()), RencodeValue::Int(10));
        inner.insert(
            RencodeValue::Str("y".into()),
            RencodeValue::Str("hi".into()),
        );
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("Foo".into()), RencodeValue::Dict(inner));
        let value = RencodeValue::Dict(map);

        let result: MyEnum = MyEnum::deserialize(&value).expect("deserialize");

        assert_eq!(
            result,
            MyEnum::Foo {
                x: 10,
                y: "hi".into(),
            }
        );
    }

    #[test]
    fn when_deserialize_enum_tuple_variant_then_works() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum MyEnum {
            Foo(i64, String),
        }

        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("Foo".into()),
            RencodeValue::List(vec![RencodeValue::Int(10), RencodeValue::Str("hi".into())]),
        );
        let value = RencodeValue::Dict(map);

        let result: MyEnum = MyEnum::deserialize(&value).expect("deserialize");

        assert_eq!(result, MyEnum::Foo(10, "hi".into()));
    }

    #[test]
    #[expect(
        clippy::empty_enum_variants_with_brackets,
        reason = "test exercises empty tuple variant deserialization"
    )]
    fn when_deserialize_enum_empty_tuple_variant_then_works() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum MyEnum {
            Foo(),
        }

        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("Foo".into()), RencodeValue::List(vec![]));
        let value = RencodeValue::Dict(map);

        let result: MyEnum = MyEnum::deserialize(&value).expect("deserialize");

        assert_eq!(result, MyEnum::Foo());
    }

    #[test]
    fn when_deserialize_enum_unit_variant_from_dict_then_works() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum MyEnum {
            Foo,
        }

        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("Foo".into()), RencodeValue::None);
        let value = RencodeValue::Dict(map);

        let result: MyEnum = MyEnum::deserialize(&value).expect("deserialize");

        assert_eq!(result, MyEnum::Foo);
    }

    #[test]
    fn when_deserialize_enum_with_int_key_then_works() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum MyEnum {
            Foo(i64),
        }

        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("Foo".into()), RencodeValue::Int(42));
        let value = RencodeValue::Dict(map);

        let result: MyEnum = MyEnum::deserialize(&value).expect("deserialize");

        assert_eq!(result, MyEnum::Foo(42));
    }

    #[test]
    fn when_deserialize_rencode_value_from_rencode_value_then_roundtrips() {
        let original = RencodeValue::Dict(make_test_dict().as_dict().unwrap().clone());

        let result: RencodeValue = RencodeValue::deserialize(&original).expect("deserialize");

        assert_eq!(result, original);
    }

    #[test]
    fn when_deserialize_rencode_value_from_rencode_value_owned_then_roundtrips() {
        let original = RencodeValue::Dict(make_test_dict().as_dict().unwrap().clone());

        let result: RencodeValue =
            RencodeValue::deserialize(original.clone()).expect("deserialize");

        assert_eq!(result, original);
    }
}
