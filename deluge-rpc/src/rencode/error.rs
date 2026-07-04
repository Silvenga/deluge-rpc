/// Errors produced by [`decode`] and typed accessors.
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
    #[error("missing field `{0}`")]
    MissingField(String),
    #[error("field `{field}` expected {expected}, got {got}")]
    WrongType {
        field: String,
        expected: &'static str,
        got: &'static str,
    },
    #[error("{0}")]
    Message(String),
    #[error("unknown field `{0}`")]
    UnknownField(String),
    #[error("duplicate field `{0}`")]
    DuplicateField(String),
    #[error("invalid tagged JSON: {0}")]
    InvalidTaggedJson(String),
}

use serde::de::Error as SerdeDeError;
use serde::de::{Expected, Unexpected};
use std::fmt::Display;

impl SerdeDeError for RencodeError {
    fn custom<T: Display>(msg: T) -> Self {
        RencodeError::Message(msg.to_string())
    }

    fn invalid_type(unexp: Unexpected, exp: &dyn Expected) -> Self {
        RencodeError::Message(format!("invalid type: {unexp}, expected {exp}"))
    }

    fn invalid_value(unexp: Unexpected, exp: &dyn Expected) -> Self {
        RencodeError::Message(format!("invalid value: {unexp}, expected {exp}"))
    }

    fn invalid_length(len: usize, exp: &dyn Expected) -> Self {
        RencodeError::Message(format!("invalid length {len}, expected {exp}"))
    }

    fn unknown_field(field: &str, _expected: &'static [&'static str]) -> Self {
        RencodeError::UnknownField(field.to_owned())
    }

    fn missing_field(field: &'static str) -> Self {
        RencodeError::MissingField(field.to_owned())
    }

    fn duplicate_field(field: &'static str) -> Self {
        RencodeError::DuplicateField(field.to_owned())
    }
}
