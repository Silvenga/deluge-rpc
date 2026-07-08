use crate::{DelugeRpcError, DelugeRpcRequest, RencodeValue};

/// A recorded request-response interaction with the Deluge daemon.
pub struct RecordedInteraction {
    /// The RPC request that was sent.
    pub request: DelugeRpcRequest,
    /// The response received from the daemon.
    pub response: RecordedResponse,
}

/// A recorded response from the Deluge daemon, either success or RPC error.
pub enum RecordedResponse {
    /// A successful RPC response.
    Ok {
        /// The decoded return value.
        value: RencodeValue,
    },
    /// An RPC error response.
    Error {
        /// The Python exception type name.
        exc_type: String,
        /// The exception message.
        exc_msg: String,
        /// The Python traceback string.
        traceback: String,
    },
}

impl RecordedInteraction {
    /// Create a `RecordedInteraction` from a request and its result.
    /// Returns `None` for non-RPC errors (e.g. transport errors).
    pub fn from_result(
        request: DelugeRpcRequest,
        result: &Result<RencodeValue, DelugeRpcError>,
    ) -> Option<Self> {
        match result {
            Ok(value) => Some(Self {
                request,
                response: RecordedResponse::Ok {
                    value: value.clone(),
                },
            }),
            Err(DelugeRpcError::RpcError {
                exc_type,
                exc_msg,
                traceback,
            }) => Some(Self {
                request,
                response: RecordedResponse::Error {
                    exc_type: exc_type.clone(),
                    exc_msg: exc_msg.clone(),
                    traceback: traceback.clone(),
                },
            }),
            Err(_) => None,
        }
    }
}
