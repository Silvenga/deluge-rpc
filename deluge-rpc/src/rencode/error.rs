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
}
