use crate::RencodeValue;
use crate::protocol::ProtocolError;
use crate::transport::TransportError;
use deluge_rpc_rencode::RencodeError;

/// Top-level errors from the Deluge RPC client.
#[derive(Debug, thiserror::Error)]
pub enum DelugeRpcError {
    /// Daemon returned an RPC error response (tag `2`).
    #[error("daemon RPC error ({exc_type}): {exc_msg}\ntraceback: {traceback}")]
    RpcError {
        /// Python exception class name.
        exc_type: String,
        /// Human-readable exception message.
        exc_msg: String,
        /// Python traceback string for debugging.
        traceback: String,
    },
    /// Connection closed while waiting for an RPC response.
    #[error("connection closed while waiting for RPC response `{method}`")]
    ConnectionClosed {
        /// Method being called when the connection was closed.
        method: String,
    },
    /// Connection already closed.
    #[error("connection already closed")]
    NotConnected,
    /// Timed out waiting for an RPC response.
    #[error("timed out waiting for RPC response")]
    Timeout,
    /// Transport-level error.
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),
    /// Protocol-level error during message parsing.
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    /// Failed to deserialize a rencode response.
    #[error("failed to deserialize response: {0}")]
    Deserialization(#[from] RencodeError),
    /// Method returned an unexpected type.
    #[error("{method} returned unexpected type: {value:?}")]
    UnexpectedResponseType {
        /// Method name.
        method: String,
        /// Unexpected return value.
        value: RencodeValue,
    },
}
