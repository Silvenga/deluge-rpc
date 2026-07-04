//! Clean-room implementation based on the rencode format specification. No GPL code was referenced.
//!
//! rencode is a compact serialization format similar to bencode, used by the
//! Deluge daemon RPC wire protocol. This module implements the subset of the
//! format needed for Deluge RPC: `None`, bool, int, str/bytes, list, dict, and
//! floats (f32 on encode, both f32 and f64 on decode).
//!
//! Reference: the rencode SPEC.md (aresch/rencode) and the public-domain
//! dart-rencode (teal77/dart-rencode, Unlicense) for the Deluge constant set.
//! No GPL-licensed Python/Cython/Go source was referenced.

#![expect(
    dead_code,
    reason = "rencode codec is the foundation for daemon RPC client in tasks 2, 3, 7"
)]

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::str::from_utf8;

// ---------------------------------------------------------------------------
// Constants — rencode type codes (Deluge / standard rencode).
// ---------------------------------------------------------------------------

/// Variable-length list prefix (`0x3B`). Followed by elements and `CHR_TERM`.
const CHR_LIST: u8 = 59;
/// Variable-length dict prefix (`0x3C`). Followed by key-value pairs and `CHR_TERM`.
const CHR_DICT: u8 = 60;
/// Big-number ASCII prefix (`0x3D`). Followed by ASCII digits and `CHR_TERM`.
const CHR_INT: u8 = 61;
/// 1-byte signed int prefix (`0x3E`).
const CHR_INT1: u8 = 62;
/// 2-byte signed int prefix (`0x3F`).
const CHR_INT2: u8 = 63;
/// 4-byte signed int prefix (`0x40`).
const CHR_INT4: u8 = 64;
/// 8-byte signed int prefix (`0x41`).
const CHR_INT8: u8 = 65;
/// 32-bit float prefix (`0x42`).
const CHR_FLOAT32: u8 = 66;
/// 64-bit float prefix (`0x2C`).
const CHR_FLOAT64: u8 = 44;
/// `True` (`0x43`).
const CHR_TRUE: u8 = 67;
/// `False` (`0x44`).
const CHR_FALSE: u8 = 68;
/// `None` (`0x45`).
const CHR_NONE: u8 = 69;
/// Terminator for variable-length lists, dicts, and big numbers (`0x7F`).
const CHR_TERM: u8 = 127;
/// Length delimiter for long strings (`:`).
const LENGTH_DELIM: u8 = b':';

/// Start of embedded positive int range (`0x00`). Values `0..INT_POS_FIXED_COUNT`.
const INT_POS_FIXED_START: u8 = 0;
/// Count of embedded positive ints (44). So `0x00..=0x2B` encode `0..=43`.
const INT_POS_FIXED_COUNT: u8 = 44;
/// Start of embedded negative int range (`0x46`). Values `-1..-INT_NEG_FIXED_COUNT`.
const INT_NEG_FIXED_START: u8 = 70;
/// Count of embedded negative ints (32). So `0x46..=0x65` encode `-1..=-32`.
const INT_NEG_FIXED_COUNT: u8 = 32;

/// Start of fixed-length dict range (`0x66`). Dicts with `1..DICT_FIXED_COUNT` entries.
const DICT_FIXED_START: u8 = 102;
/// Count of fixed-length dicts (25). So `0x66..=0x7E` encode dicts of size `1..=25`.
const DICT_FIXED_COUNT: u8 = 25;

/// Start of fixed-length string range (`0x80`). Strings of length `0..STR_FIXED_COUNT`.
const STR_FIXED_START: u8 = 128;
/// Count of fixed-length strings (64). So `0x80..=0xBF` encode strings of length `0..=63`.
const STR_FIXED_COUNT: u8 = 64;

/// Start of fixed-length list range (`0xC0`). Lists with `0..LIST_FIXED_COUNT` entries.
const LIST_FIXED_START: u8 = STR_FIXED_START + STR_FIXED_COUNT;
/// Count of fixed-length lists (64). So `0xC0..=0xFF` encode lists of size `0..=63`.
const LIST_FIXED_COUNT: u8 = 64;

/// Maximum decode nesting depth before `DepthExceeded` is returned.
const MAX_DEPTH: usize = 100;

// ---------------------------------------------------------------------------
// Error type.
// ---------------------------------------------------------------------------

/// Errors produced by [`decode`].
#[derive(Debug, thiserror::Error)]
pub enum RencodeError {
    #[error("invalid byte at offset {0}: 0x{1:02x}")]
    InvalidByte(usize, u8),
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("decode depth exceeded limit ({0})")]
    DepthExceeded(usize),
    #[error("invalid UTF-8 in string")]
    InvalidUtf8,
    #[error("number parse error: {0}")]
    NumberParse(String),
}

// ---------------------------------------------------------------------------
// Value type.
// ---------------------------------------------------------------------------

/// A rencode value covering all types needed for Deluge daemon RPC.
///
/// `Ord` is implemented so that [`RencodeValue`] can be used as a `BTreeMap`
/// key, giving deterministic dict encoding order. Floats are ordered by their
/// bit patterns (via `to_bits`), which is total and deterministic even though
/// it is not numerical order — acceptable since floats are never used as dict
/// keys in Deluge RPC.
#[derive(Debug, Clone, PartialEq)]
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

impl PartialOrd for RencodeValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
        // Different variants are already disambiguated by discriminant_tag.
        _ => Ordering::Equal,
    }
}

// ---------------------------------------------------------------------------
// Encode.
// ---------------------------------------------------------------------------

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

#[expect(
    clippy::as_conversions,
    reason = "rencode int encoding requires explicit width-narrowing casts after range checks"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "range checks guarantee the value fits the target width"
)]
#[expect(
    clippy::cast_sign_loss,
    reason = "negative ints are encoded as unsigned byte offsets after range check"
)]
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

#[expect(
    clippy::as_conversions,
    reason = "length is checked < STR_FIXED_COUNT (64) before cast to u8"
)]
#[expect(clippy::cast_possible_truncation, reason = "len < 64 fits in u8")]
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

#[expect(
    clippy::as_conversions,
    reason = "length is checked < LIST_FIXED_COUNT (64) before cast to u8"
)]
#[expect(clippy::cast_possible_truncation, reason = "len < 64 fits in u8")]
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

#[expect(
    clippy::as_conversions,
    reason = "length is checked < DICT_FIXED_COUNT (25) before cast to u8"
)]
#[expect(clippy::cast_possible_truncation, reason = "len < 25 fits in u8")]
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

#[expect(
    clippy::as_conversions,
    reason = "Deluge daemon protocol encodes floats as f32 by design"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64 to f32 is the intended rencode encoding"
)]
fn encode_float32(f: f64, out: &mut Vec<u8>) {
    out.push(CHR_FLOAT32);
    out.extend_from_slice(&(f as f32).to_be_bytes());
}

// ---------------------------------------------------------------------------
// Decode.
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Classification helpers.
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Cursor — a tiny bounds-checked reader over a byte slice.
// ---------------------------------------------------------------------------

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_byte(&mut self) -> Option<u8> {
        let b = *self.data.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }

    fn peek_byte(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    fn advance(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n).min(self.data.len());
    }

    fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        if end > self.data.len() {
            return None;
        }
        let slice = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    #[expect(clippy::as_conversions, reason = "u8 to i8 is a sign reinterpretation")]
    #[expect(
        clippy::cast_possible_wrap,
        reason = "u8 to i8 wrap is the intended signed reinterpretation"
    )]
    fn read_i8(&mut self) -> Option<i8> {
        let b = self.read_byte()?;
        Some(b as i8)
    }

    fn read_i16(&mut self) -> Option<i16> {
        let bytes: [u8; 2] = self.read_bytes(2)?.try_into().ok()?;
        Some(i16::from_be_bytes(bytes))
    }

    fn read_i32(&mut self) -> Option<i32> {
        let bytes: [u8; 4] = self.read_bytes(4)?.try_into().ok()?;
        Some(i32::from_be_bytes(bytes))
    }

    fn read_i64(&mut self) -> Option<i64> {
        let bytes: [u8; 8] = self.read_bytes(8)?.try_into().ok()?;
        Some(i64::from_be_bytes(bytes))
    }

    fn read_f32(&mut self) -> Option<f32> {
        let bytes: [u8; 4] = self.read_bytes(4)?.try_into().ok()?;
        Some(f32::from_be_bytes(bytes))
    }

    fn read_f64(&mut self) -> Option<f64> {
        let bytes: [u8; 8] = self.read_bytes(8)?.try_into().ok()?;
        Some(f64::from_be_bytes(bytes))
    }
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for clarity on shape errors"
)]
#[expect(
    clippy::unwrap_used,
    reason = "tests use unwrap for clarity on known-good values"
)]
#[expect(
    clippy::indexing_slicing,
    reason = "tests index known-length encoded buffers"
)]
#[expect(
    clippy::as_conversions,
    reason = "tests cast small known lengths to u8"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "test lengths are small and fit u8"
)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

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
        let decoded = decode(&encoded).expect("roundtrip");
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
        let expected = (1234.5_f32).to_be_bytes();
        assert_eq!(&encoded[1..], expected);
    }

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
