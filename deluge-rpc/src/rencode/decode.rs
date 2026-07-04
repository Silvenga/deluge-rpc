use crate::rencode::constants::{
    CHR_DICT, CHR_FALSE, CHR_FLOAT32, CHR_FLOAT64, CHR_INT, CHR_INT1, CHR_INT2, CHR_INT4, CHR_INT8,
    CHR_LIST, CHR_NONE, CHR_TERM, CHR_TRUE, DICT_FIXED_COUNT, DICT_FIXED_START,
    INT_NEG_FIXED_COUNT, INT_NEG_FIXED_START, INT_POS_FIXED_COUNT, INT_POS_FIXED_START,
    LENGTH_DELIM, LIST_FIXED_COUNT, LIST_FIXED_START, MAX_DEPTH, STR_FIXED_COUNT, STR_FIXED_START,
};
use crate::rencode::cursor::Cursor;
use crate::rencode::error::RencodeError;
use crate::rencode::value::RencodeValue;
use std::collections::BTreeMap;
use std::str::from_utf8;

/// Decode a rencode byte stream into a [`RencodeValue`].
///
/// Enforces a nesting depth limit of [`MAX_DEPTH`] to prevent stack overflow
/// on malicious input.
pub fn decode(data: &[u8]) -> Result<RencodeValue, RencodeError> {
    let mut cursor = Cursor::new(data);
    decode_inner(&mut cursor, 0)
}

fn decode_inner(cursor: &mut Cursor<'_>, depth: usize) -> Result<RencodeValue, RencodeError> {
    if depth > MAX_DEPTH {
        return Err(RencodeError::DepthExceeded(MAX_DEPTH));
    }

    let offset = cursor.pos;
    let byte = cursor.read_byte().ok_or(RencodeError::UnexpectedEof)?;

    match byte {
        CHR_NONE => Ok(RencodeValue::None),
        CHR_TRUE => Ok(RencodeValue::Bool(true)),
        CHR_FALSE => Ok(RencodeValue::Bool(false)),
        CHR_INT1 => {
            let v = cursor.read_i8().ok_or(RencodeError::UnexpectedEof)?;
            Ok(RencodeValue::Int(i64::from(v)))
        }
        CHR_INT2 => {
            let v = cursor.read_i16().ok_or(RencodeError::UnexpectedEof)?;
            Ok(RencodeValue::Int(i64::from(v)))
        }
        CHR_INT4 => {
            let v = cursor.read_i32().ok_or(RencodeError::UnexpectedEof)?;
            Ok(RencodeValue::Int(i64::from(v)))
        }
        CHR_INT8 => {
            let v = cursor.read_i64().ok_or(RencodeError::UnexpectedEof)?;
            Ok(RencodeValue::Int(v))
        }
        CHR_INT => decode_big_number(cursor),
        CHR_FLOAT32 => {
            let v = cursor.read_f32().ok_or(RencodeError::UnexpectedEof)?;
            Ok(RencodeValue::Float(f64::from(v)))
        }
        CHR_FLOAT64 => {
            let v = cursor.read_f64().ok_or(RencodeError::UnexpectedEof)?;
            Ok(RencodeValue::Float(v))
        }
        CHR_LIST => decode_list_terminated(cursor, depth),
        CHR_DICT => decode_dict_terminated(cursor, depth),
        CHR_TERM => Err(RencodeError::InvalidByte(offset, byte)),
        b if is_fixed_pos_int(b) => Ok(RencodeValue::Int(i64::from(b - INT_POS_FIXED_START))),
        b if is_fixed_neg_int(b) => {
            let raw = i64::from(b) - i64::from(INT_NEG_FIXED_START) + 1;
            Ok(RencodeValue::Int(-raw))
        }
        b if is_fixed_string(b) => {
            let len = usize::from(b - STR_FIXED_START);
            let bytes = cursor.read_bytes(len).ok_or(RencodeError::UnexpectedEof)?;
            Ok(decode_string_or_bytes(bytes))
        }
        b if is_fixed_list(b) => {
            let len = usize::from(b - LIST_FIXED_START);
            decode_list_fixed(cursor, depth, len)
        }
        b if is_fixed_dict(b) => {
            let len = usize::from(b - DICT_FIXED_START);
            decode_dict_fixed(cursor, depth, len)
        }
        b if is_long_string_digit(b) => decode_long_string(cursor, b),
        _ => Err(RencodeError::InvalidByte(offset, byte)),
    }
}

fn decode_big_number(cursor: &mut Cursor<'_>) -> Result<RencodeValue, RencodeError> {
    let mut buf = Vec::new();
    loop {
        let b = cursor.read_byte().ok_or(RencodeError::UnexpectedEof)?;
        if b == CHR_TERM {
            break;
        }
        if !b.is_ascii_digit() && b != b'-' {
            return Err(RencodeError::NumberParse(format!(
                "invalid byte 0x{b:02x} in number"
            )));
        }
        buf.push(b);
    }
    let s = from_utf8(&buf).map_err(|_| RencodeError::NumberParse("non-utf8".into()))?;
    if let Ok(v) = s.parse::<i64>() {
        return Ok(RencodeValue::Int(v));
    }
    if let Ok(v) = s.parse::<f64>() {
        return Ok(RencodeValue::Float(v));
    }
    Err(RencodeError::NumberParse(format!("cannot parse {s}")))
}

fn decode_list_fixed(
    cursor: &mut Cursor<'_>,
    depth: usize,
    len: usize,
) -> Result<RencodeValue, RencodeError> {
    let mut items = Vec::with_capacity(len);
    for _ in 0..len {
        items.push(decode_inner(cursor, depth + 1)?);
    }
    Ok(RencodeValue::List(items))
}

fn decode_list_terminated(
    cursor: &mut Cursor<'_>,
    depth: usize,
) -> Result<RencodeValue, RencodeError> {
    let mut items = Vec::new();
    loop {
        let peek = cursor.peek_byte().ok_or(RencodeError::UnexpectedEof)?;
        if peek == CHR_TERM {
            cursor.advance(1);
            return Ok(RencodeValue::List(items));
        }
        items.push(decode_inner(cursor, depth + 1)?);
    }
}

fn decode_dict_fixed(
    cursor: &mut Cursor<'_>,
    depth: usize,
    len: usize,
) -> Result<RencodeValue, RencodeError> {
    let mut map = BTreeMap::new();
    for _ in 0..len {
        let key = decode_inner(cursor, depth + 1)?;
        let val = decode_inner(cursor, depth + 1)?;
        map.insert(key, val);
    }
    Ok(RencodeValue::Dict(map))
}

fn decode_dict_terminated(
    cursor: &mut Cursor<'_>,
    depth: usize,
) -> Result<RencodeValue, RencodeError> {
    let mut map = BTreeMap::new();
    loop {
        let peek = cursor.peek_byte().ok_or(RencodeError::UnexpectedEof)?;
        if peek == CHR_TERM {
            cursor.advance(1);
            return Ok(RencodeValue::Dict(map));
        }
        let key = decode_inner(cursor, depth + 1)?;
        let val = decode_inner(cursor, depth + 1)?;
        map.insert(key, val);
    }
}

fn decode_long_string(
    cursor: &mut Cursor<'_>,
    first_digit: u8,
) -> Result<RencodeValue, RencodeError> {
    let mut len_buf = Vec::new();
    len_buf.push(first_digit);
    loop {
        let b = cursor.read_byte().ok_or(RencodeError::UnexpectedEof)?;
        if b == LENGTH_DELIM {
            break;
        }
        if !b.is_ascii_digit() {
            return Err(RencodeError::InvalidByte(cursor.pos.saturating_sub(1), b));
        }
        len_buf.push(b);
    }
    let len_str = from_utf8(&len_buf).map_err(|_| RencodeError::UnexpectedEof)?;
    let len: usize = len_str
        .parse()
        .map_err(|_| RencodeError::NumberParse(format!("invalid string length {len_str}")))?;
    let bytes = cursor.read_bytes(len).ok_or(RencodeError::UnexpectedEof)?;
    Ok(decode_string_or_bytes(bytes))
}

fn decode_string_or_bytes(bytes: &[u8]) -> RencodeValue {
    match from_utf8(bytes) {
        Ok(s) => RencodeValue::Str(s.to_owned()),
        Err(_) => RencodeValue::Bytes(bytes.to_owned()),
    }
}

fn is_fixed_pos_int(b: u8) -> bool {
    (INT_POS_FIXED_START..INT_POS_FIXED_START + INT_POS_FIXED_COUNT).contains(&b)
}

fn is_fixed_neg_int(b: u8) -> bool {
    (INT_NEG_FIXED_START..INT_NEG_FIXED_START + INT_NEG_FIXED_COUNT).contains(&b)
}

fn is_fixed_string(b: u8) -> bool {
    (STR_FIXED_START..STR_FIXED_START + STR_FIXED_COUNT).contains(&b)
}

fn is_fixed_list(b: u8) -> bool {
    b >= LIST_FIXED_START && b.wrapping_sub(LIST_FIXED_START) < LIST_FIXED_COUNT
}

fn is_fixed_dict(b: u8) -> bool {
    (DICT_FIXED_START..DICT_FIXED_START + DICT_FIXED_COUNT).contains(&b)
}

fn is_long_string_digit(b: u8) -> bool {
    (b'1'..=b'9').contains(&b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::constants::{CHR_FLOAT64, CHR_INT};
    use crate::rencode::encode::encode;

    #[test]
    fn when_roundtrip_all_types_then_decode_matches_encode() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str(String::from("key")),
            RencodeValue::Int(42),
        );
        map.insert(
            RencodeValue::Str(String::from("nested")),
            RencodeValue::List(vec![RencodeValue::Bool(true), RencodeValue::None]),
        );

        let original = RencodeValue::List(vec![
            RencodeValue::None,
            RencodeValue::Bool(true),
            RencodeValue::Bool(false),
            RencodeValue::Int(0),
            RencodeValue::Int(43),
            RencodeValue::Int(-1),
            RencodeValue::Int(-32),
            RencodeValue::Int(127),
            RencodeValue::Int(-128),
            RencodeValue::Int(32_767),
            RencodeValue::Int(-32_768),
            RencodeValue::Int(2_147_483_647),
            RencodeValue::Int(-2_147_483_648),
            RencodeValue::Int(i64::MAX),
            RencodeValue::Int(i64::MIN),
            RencodeValue::Str(String::from("hello")),
            RencodeValue::Str(String::new()),
            RencodeValue::Bytes(vec![0x00, 0xFF, 0x80]),
            RencodeValue::List(Vec::new()),
            RencodeValue::Dict(BTreeMap::new()),
            RencodeValue::Dict(map),
            RencodeValue::Float(f64::from(1.5_f32)),
        ]);

        let encoded = encode(&original);
        let decoded = decode(&encoded).expect("roundtrip decode");

        assert_eq!(decoded, original);
    }

    #[test]
    fn when_decode_f64_then_converts_to_f64() {
        let value = 1.1_f64;
        let mut encoded = vec![CHR_FLOAT64];
        encoded.extend_from_slice(&value.to_be_bytes());

        let decoded = decode(&encoded).expect("f64 decode");

        assert_eq!(decoded, RencodeValue::Float(value));
    }

    #[test]
    fn when_decode_truncated_input_then_returns_unexpected_eof() {
        let encoded = vec![CHR_INT4];

        let result = decode(&encoded);

        assert!(matches!(result, Err(RencodeError::UnexpectedEof)));
    }

    #[test]
    fn when_decode_deeply_nested_then_returns_depth_exceeded() {
        let depth = MAX_DEPTH + 5;
        let mut encoded = Vec::new();
        for _ in 0..depth {
            encoded.push(LIST_FIXED_START + 1);
        }
        encoded.push(0x01);

        let result = decode(&encoded);

        assert!(matches!(result, Err(RencodeError::DepthExceeded(_))));
    }

    #[test]
    fn when_decode_invalid_byte_then_returns_invalid_byte_error() {
        let encoded = vec![0x2E];

        let result = decode(&encoded);

        assert!(matches!(result, Err(RencodeError::InvalidByte(0, 0x2E))));
    }

    #[test]
    fn when_encode_long_string_then_length_delimited_encoding() {
        let s = String::from("a").repeat(100);

        let encoded = encode(&RencodeValue::Str(s.clone()));

        assert!(encoded.starts_with(b"100:"));
        assert_eq!(encoded.len(), 4 + 100);

        let decoded = decode(&encoded).expect("long string roundtrip");
        assert_eq!(decoded, RencodeValue::Str(s));
    }

    #[test]
    fn when_encode_long_list_then_terminated_form() {
        let items: Vec<RencodeValue> = (0..70).map(RencodeValue::Int).collect();

        let encoded = encode(&RencodeValue::List(items.clone()));

        assert_eq!(encoded[0], CHR_LIST);
        assert_eq!(*encoded.last().unwrap(), CHR_TERM);

        let decoded = decode(&encoded).expect("long list roundtrip");
        assert_eq!(decoded, RencodeValue::List(items));
    }

    #[test]
    fn when_encode_long_dict_then_terminated_form() {
        let mut map = BTreeMap::new();
        for i in 0..30 {
            map.insert(RencodeValue::Int(i), RencodeValue::Int(i * 2));
        }

        let encoded = encode(&RencodeValue::Dict(map.clone()));

        assert_eq!(encoded[0], CHR_DICT);
        assert_eq!(*encoded.last().unwrap(), CHR_TERM);

        let decoded = decode(&encoded).expect("long dict roundtrip");
        assert_eq!(decoded, RencodeValue::Dict(map));
    }

    #[test]
    fn when_decode_big_number_ascii_then_returns_int() {
        let mut encoded = vec![CHR_INT];
        encoded.extend_from_slice(b"12345678901234");
        encoded.push(CHR_TERM);

        let decoded = decode(&encoded).expect("big number decode");

        assert_eq!(decoded, RencodeValue::Int(12_345_678_901_234));
    }

    #[test]
    fn when_decode_non_utf8_string_then_returns_bytes() {
        let raw = vec![0xFF, 0xFE, 0xFD];
        let mut encoded = vec![STR_FIXED_START + raw.len() as u8];
        encoded.extend_from_slice(&raw);

        let decoded = decode(&encoded).expect("bytes decode");

        assert_eq!(decoded, RencodeValue::Bytes(raw));
    }
}
