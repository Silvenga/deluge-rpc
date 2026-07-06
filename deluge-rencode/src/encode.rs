use crate::constants::{
    CHR_DICT, CHR_FALSE, CHR_FLOAT32, CHR_INT1, CHR_INT2, CHR_INT4, CHR_INT8, CHR_LIST, CHR_NONE,
    CHR_TERM, CHR_TRUE, DICT_FIXED_COUNT, DICT_FIXED_START, INT_NEG_FIXED_COUNT,
    INT_NEG_FIXED_START, INT_POS_FIXED_COUNT, INT_POS_FIXED_START, LENGTH_DELIM, LIST_FIXED_COUNT,
    LIST_FIXED_START, STR_FIXED_COUNT, STR_FIXED_START,
};
use crate::value::RencodeValue;
use std::collections::BTreeMap;
use std::fmt::Write;

/// Encode a [`RencodeValue`] into the rencode wire format.
///
/// Floats are encoded as 32-bit (Deluge default). Dicts are emitted in
/// `BTreeMap` iteration order, which is deterministic.
#[must_use]
pub fn encode(value: &RencodeValue) -> Vec<u8> {
    let mut out = Vec::new();
    encode_into(value, &mut out);
    out
}

fn encode_into(value: &RencodeValue, out: &mut Vec<u8>) {
    match value {
        RencodeValue::None => out.push(CHR_NONE),
        RencodeValue::Bool(true) => out.push(CHR_TRUE),
        RencodeValue::Bool(false) => out.push(CHR_FALSE),
        RencodeValue::Int(i) => encode_int(*i, out),
        RencodeValue::Str(s) => encode_bytes(s.as_bytes(), out),
        RencodeValue::Bytes(b) => encode_bytes(b, out),
        RencodeValue::List(items) => encode_list(items, out),
        RencodeValue::Dict(map) => encode_dict(map, out),
        RencodeValue::Float(f) => encode_float32(*f, out),
    }
}

fn encode_int(i: i64, out: &mut Vec<u8>) {
    if (0..i64::from(INT_POS_FIXED_COUNT)).contains(&i) {
        out.push(INT_POS_FIXED_START.wrapping_add(i as u8));
    } else if (-i64::from(INT_NEG_FIXED_COUNT)..0).contains(&i) {
        // i is in [-32, -1], so (INT_NEG_FIXED_START - 1 - i) is in [70, 101].
        let code = (i64::from(INT_NEG_FIXED_START) - 1 - i) as u8;
        out.push(code);
    } else if let Ok(v) = i8::try_from(i) {
        out.push(CHR_INT1);
        out.push(v as u8);
    } else if let Ok(v) = i16::try_from(i) {
        out.push(CHR_INT2);
        out.extend_from_slice(&v.to_be_bytes());
    } else if let Ok(v) = i32::try_from(i) {
        out.push(CHR_INT4);
        out.extend_from_slice(&v.to_be_bytes());
    } else {
        out.push(CHR_INT8);
        out.extend_from_slice(&i.to_be_bytes());
    }
}

fn encode_bytes(bytes: &[u8], out: &mut Vec<u8>) {
    let len = bytes.len();
    if len < usize::from(STR_FIXED_COUNT) {
        out.push(STR_FIXED_START.wrapping_add(len as u8));
        out.extend_from_slice(bytes);
    } else {
        let mut len_str = String::new();
        let _ = write!(len_str, "{len}");
        out.extend_from_slice(len_str.as_bytes());
        out.push(LENGTH_DELIM);
        out.extend_from_slice(bytes);
    }
}

fn encode_list(items: &[RencodeValue], out: &mut Vec<u8>) {
    let len = items.len();
    if len < usize::from(LIST_FIXED_COUNT) {
        out.push(LIST_FIXED_START.wrapping_add(len as u8));
        for item in items {
            encode_into(item, out);
        }
    } else {
        out.push(CHR_LIST);
        for item in items {
            encode_into(item, out);
        }
        out.push(CHR_TERM);
    }
}

fn encode_dict(map: &BTreeMap<RencodeValue, RencodeValue>, out: &mut Vec<u8>) {
    let len = map.len();
    if len < usize::from(DICT_FIXED_COUNT) {
        out.push(DICT_FIXED_START.wrapping_add(len as u8));
        for (k, v) in map {
            encode_into(k, out);
            encode_into(v, out);
        }
    } else {
        out.push(CHR_DICT);
        for (k, v) in map {
            encode_into(k, out);
            encode_into(v, out);
        }
        out.push(CHR_TERM);
    }
}

fn encode_float32(f: f64, out: &mut Vec<u8>) {
    out.push(CHR_FLOAT32);
    out.extend_from_slice(&(f as f32).to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_encode_none_then_single_byte_0x45() {
        let encoded = encode(&RencodeValue::None);

        assert_eq!(encoded, vec![0x45]);
    }

    #[test]
    fn when_encode_bool_true_then_byte_0x43() {
        let encoded = encode(&RencodeValue::Bool(true));

        assert_eq!(encoded, vec![0x43]);
    }

    #[test]
    fn when_encode_bool_false_then_byte_0x44() {
        let encoded = encode(&RencodeValue::Bool(false));

        assert_eq!(encoded, vec![0x44]);
    }

    #[test]
    fn when_encode_small_positive_int_then_single_byte() {
        let encoded = encode(&RencodeValue::Int(5));

        assert_eq!(encoded, vec![0x05]);
    }

    #[test]
    fn when_encode_small_negative_int_then_single_byte() {
        let encoded = encode(&RencodeValue::Int(-10));

        assert_eq!(encoded, vec![0x4F]);
    }

    #[test]
    fn when_encode_large_int_then_multi_byte() {
        let value = 1_000_000_i64;

        let encoded = encode(&RencodeValue::Int(value));

        assert_eq!(encoded[0], CHR_INT4);
        assert_eq!(encoded.len(), 5);
        let decoded = RencodeValue::decode(&encoded).expect("roundtrip");
        assert_eq!(decoded, RencodeValue::Int(value));
    }

    #[test]
    fn when_encode_string_then_length_prefixed() {
        let encoded = encode(&RencodeValue::Str(String::from("foobar")));

        assert_eq!(encoded, [0x86, b'f', b'o', b'o', b'b', b'a', b'r']);
    }

    #[test]
    fn when_encode_empty_string_then_correct_bytes() {
        let encoded = encode(&RencodeValue::Str(String::new()));

        assert_eq!(encoded, vec![STR_FIXED_START]);
    }

    #[test]
    fn when_encode_list_then_length_prefixed() {
        let list = RencodeValue::List(vec![
            RencodeValue::Int(1),
            RencodeValue::Int(2),
            RencodeValue::Int(3),
        ]);

        let encoded = encode(&list);

        assert_eq!(encoded, [0xC3, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn when_encode_empty_list_then_correct_bytes() {
        let encoded = encode(&RencodeValue::List(Vec::new()));

        assert_eq!(encoded, vec![LIST_FIXED_START]);
    }

    #[test]
    fn when_encode_dict_then_btreemap_order() {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str(String::from("b")), RencodeValue::Int(2));
        map.insert(RencodeValue::Str(String::from("a")), RencodeValue::Int(1));

        let encoded = encode(&RencodeValue::Dict(map));

        assert_eq!(encoded, [0x68, 0x81, b'a', 0x01, 0x81, b'b', 0x02]);
    }

    #[test]
    fn when_encode_empty_dict_then_correct_bytes() {
        let encoded = encode(&RencodeValue::Dict(BTreeMap::new()));

        assert_eq!(encoded, vec![DICT_FIXED_START]);
    }

    #[test]
    fn when_encode_float_then_f32_encoding() {
        let encoded = encode(&RencodeValue::Float(1234.5_f64));

        assert_eq!(encoded[0], CHR_FLOAT32);
        assert_eq!(encoded.len(), 5);
        let expected = 1234.5_f32.to_be_bytes();
        assert_eq!(&encoded[1..], expected);
    }
}
