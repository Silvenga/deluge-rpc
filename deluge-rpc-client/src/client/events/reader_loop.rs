use crate::DelugeRpcError;
use crate::client::connection::Connection;
use crate::client::events::parse_event::parse_event;
use crate::protocol::DelugeRpcMessage;
use deluge_rpc_models::DelugeEvent;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc;

pub async fn reader_loop(
    connection: Arc<Connection>,
    tx: mpsc::Sender<Result<DelugeEvent, DelugeRpcError>>,
) {
    let message_queue = match connection.message_queue() {
        Some(q) => q,
        None => {
            let _ = tx.send(Err(DelugeRpcError::NotConnected)).await;
            return;
        }
    };

    let mut rx = message_queue.subscribe();

    loop {
        match rx.recv().await {
            Ok(DelugeRpcMessage::Event { name, args }) => {
                let event = parse_event(&name, &args);
                if tx.send(Ok(event)).await.is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(RecvError::Lagged(n)) => {
                tracing::warn!(n, "event subscriber lagged, missed events");
                continue;
            }
            Err(RecvError::Closed) => {
                let _ = tx
                    .send(Err(DelugeRpcError::ConnectionClosed {
                        method: "event_stream".into(),
                    }))
                    .await;
                break;
            }
        }
    }
}
