use serde::de::Error as SerdeDeError;
use serde::de::{Expected, Unexpected};
use serde::ser::Error as SerError;
use std::fmt::Display;

/// Errors produced by decoding and typed accessors.
#[derive(Debug, thiserror::Error)]
pub enum RencodeError {
    /// An unexpected byte was encountered at a given offset.
    #[error("invalid byte at offset {0}: 0x{1:02x}")]
    InvalidByte(usize, u8),
    /// The input ended before decoding was complete.
    #[error("unexpected end of input")]
    UnexpectedEof,
    /// Decode recursion depth exceeded the configured limit.
    #[error("decode depth exceeded limit ({0})")]
    DepthExceeded(usize),
    /// Failed to parse a numeric value from the input.
    #[error("number parse error: {0}")]
    NumberParse(String),
    /// A required field was not found in a dict.
    #[error("missing field `{0}`")]
    MissingField(String),
    /// A field had an unexpected type.
    #[error("field `{field}` expected {expected}, got {got}")]
    WrongType {
        /// The name of the field.
        field: String,
        /// The expected type name.
        expected: &'static str,
        /// The actual type name encountered.
        got: &'static str,
    },
    /// A generic error message.
    #[error("{0}")]
    Message(String),
    /// An unknown field was encountered during deserialization.
    #[error("unknown field `{0}`")]
    UnknownField(String),
    /// A duplicate field was encountered during deserialization.
    #[error("duplicate field `{0}`")]
    DuplicateField(String),
    /// The tagged-JSON input was malformed.
    #[error("invalid tagged JSON: {0}")]
    InvalidTaggedJson(String),
    /// A custom serialization error.
    #[error("{0}")]
    Custom(String),
}

impl SerError for RencodeError {
    fn custom<T: Display>(msg: T) -> Self {
        RencodeError::Custom(msg.to_string())
    }
}

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
