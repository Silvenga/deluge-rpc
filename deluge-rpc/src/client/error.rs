use crate::RencodeValue;
use crate::protocol::ProtocolError;
use crate::transport::TransportError;
use deluge_rencode::RencodeError;

#[derive(Debug, thiserror::Error)]
pub enum DelugeRpcError {
    #[error("daemon RPC error ({exc_type}): {exc_msg}\ntraceback: {traceback}")]
    RpcError {
        exc_type: String,
        exc_msg: String,
        traceback: String,
    },
    #[error("connection closed while waiting for RPC response `{method}`")]
    ConnectionClosed { method: String },
    #[error("connection already closed")]
    NotConnected,
    #[error("timed out waiting for RPC response")]
    Timeout,
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("failed to deserialize response: {0}")]
    Deserialization(#[from] RencodeError),
    #[error("{method} returned unexpected type: {value:?}")]
    UnexpectedResponseType { method: String, value: RencodeValue },
}
