use crate::client::connection::Connection;
use crate::{DelugeRpcMessage, DelugeRpcRequest, RencodeValue};
use anyhow::{bail, Context};
use tokio::sync::broadcast::error::RecvError;

pub async fn dispatch(
    connection: &Connection,
    request: DelugeRpcRequest,
) -> anyhow::Result<RencodeValue> {
    let method = request.method.clone();
    let (writer, mut rx) = connection.writer_and_rx()?;

    let id = connection.next_id();
    {
        let rencode_value = request.into_rencode_value(id);
        let encoded = rencode_value.encode();

        let mut writer = writer.lock().await;
        writer
            .send(&encoded)
            .await
            .context("failed to write to connection")?
    }

    loop {
        match rx.recv().await {
            Ok(DelugeRpcMessage::Response { id: resp_id, value }) if resp_id == id => {
                return Ok(value);
            }
            Ok(DelugeRpcMessage::Error {
                id: err_id,
                exc_type,
                exc_msg,
                traceback,
            }) if err_id == id => {
                bail!("daemon RPC error ({exc_type}): {exc_msg}\ntraceback: {traceback}")
            }
            Ok(_) => {
                // Not a message we care about.
                continue;
            }
            Err(RecvError::Lagged(n)) => {
                tracing::warn!(n, "RPC subscriber lagged, missed messages");
                continue;
            }
            Err(RecvError::Closed) => {
                bail!("connection closed while waiting for RPC response `{method}`")
            }
        }
    }
}
