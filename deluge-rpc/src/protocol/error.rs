use crate::RencodeValue;
use deluge_rencode::RencodeError;

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("unexpected RPC envelope shape (not a list): {0:?}")]
    InvalidEnvelope(RencodeValue),
    #[error("empty RPC message")]
    EmptyMessage,
    #[error("RPC message type tag is not an int: {0:?}")]
    InvalidTypeTag(RencodeValue),
    #[error("RPC message missing type tag")]
    MissingTypeTag,
    #[error("RPC response id out of u32 range: {0}")]
    ResponseIdOutOfRange(i64),
    #[error("RPC response id is not an int: {0:?}")]
    InvalidResponseId(RencodeValue),
    #[error("RPC response missing id")]
    MissingResponseId,
    #[error("RPC response missing return value")]
    MissingReturnValue,
    #[error("RPC error id out of u32 range: {0}")]
    ErrorIdOutOfRange(i64),
    #[error("RPC error id is not an int: {0:?}")]
    InvalidErrorId(RencodeValue),
    #[error("RPC error missing id")]
    MissingErrorId,
    #[error("unexpected RPC message type {0} (expected 1/2/3)")]
    UnknownMessageType(i64),
    #[error("failed to decode rencode payload: {0}")]
    Decode(#[from] RencodeError),
    #[error("{method} returned non-int value: {value:?}")]
    NotInt { method: String, value: RencodeValue },
    #[error("{method} returned non-dict value: {value:?}")]
    NotDict { method: String, value: RencodeValue },
}
