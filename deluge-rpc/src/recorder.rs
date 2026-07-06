use crate::{DelugeRpcError, DelugeRpcRequest, RencodeValue};

pub struct RecordedInteraction {
    pub request: DelugeRpcRequest,
    pub response: RecordedResponse,
}

pub enum RecordedResponse {
    Ok {
        value: RencodeValue,
    },
    Error {
        exc_type: String,
        exc_msg: String,
        traceback: String,
    },
}

impl RecordedInteraction {
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
