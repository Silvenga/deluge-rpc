use crate::protocol::decode_message;
use crate::rencode::RencodeValue;
use crate::shared::Shared;
use crate::transport::DelugeReader;
use std::sync::Arc;

pub(crate) async fn reader_loop(mut reader: DelugeReader, shared: Arc<Shared>) {
    loop {
        match reader.recv().await {
            Ok(raw) => match RencodeValue::decode(&raw) {
                Ok(decoded) => match decode_message(&decoded) {
                    Ok(msg) => {
                        let _ = shared.message_tx.send(msg);
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "failed to decode RPC message");
                    }
                },
                Err(e) => {
                    tracing::warn!(error = %e, "failed to decode rencode payload");
                }
            },
            Err(e) => {
                tracing::info!(error = %e, "reader loop ended (connection closed)");
                break;
            }
        }
    }
}
