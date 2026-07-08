use crate::RencodeValue;
use deluge_rpc_rencode::RencodeError;

/// Protocol-level errors during RPC message parsing.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    /// Unexpected RPC envelope shape (not a list).
    #[error("unexpected RPC envelope shape (not a list): {0:?}")]
    InvalidEnvelope(RencodeValue),
    /// Empty RPC message.
    #[error("empty RPC message")]
    EmptyMessage,
    /// RPC message type tag is not an int.
    #[error("RPC message type tag is not an int: {0:?}")]
    InvalidTypeTag(RencodeValue),
    /// RPC message missing type tag.
    #[error("RPC message missing type tag")]
    MissingTypeTag,
    /// RPC response id out of u32 range.
    #[error("RPC response id out of u32 range: {0}")]
    ResponseIdOutOfRange(i64),
    /// RPC response id is not an int.
    #[error("RPC response id is not an int: {0:?}")]
    InvalidResponseId(RencodeValue),
    /// RPC response missing id.
    #[error("RPC response missing id")]
    MissingResponseId,
    /// RPC response missing return value.
    #[error("RPC response missing return value")]
    MissingReturnValue,
    /// RPC error id out of u32 range.
    #[error("RPC error id out of u32 range: {0}")]
    ErrorIdOutOfRange(i64),
    /// RPC error id is not an int.
    #[error("RPC error id is not an int: {0:?}")]
    InvalidErrorId(RencodeValue),
    /// RPC error missing id.
    #[error("RPC error missing id")]
    MissingErrorId,
    /// Unexpected RPC message type (expected 1/2/3).
    #[error("unexpected RPC message type {0} (expected 1/2/3)")]
    UnknownMessageType(i64),
    /// Failed to decode rencode payload.
    #[error("failed to decode rencode payload: {0}")]
    Decode(#[from] RencodeError),
    /// Method returned non-int value.
    #[error("{method} returned non-int value: {value:?}")]
    NotInt {
        /// Method name.
        method: String,
        /// Unexpected non-int return value.
        value: RencodeValue,
    },
    /// Method returned non-dict value.
    #[error("{method} returned non-dict value: {value:?}")]
    NotDict {
        /// Method name.
        method: String,
        /// Unexpected non-dict return value.
        value: RencodeValue,
    },
}
