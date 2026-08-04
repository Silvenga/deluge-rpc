use crate::transport::{DelugeTransport, DelugeWriter, TransportError};
use crate::{DelugeRpcError, DelugeRpcMessage, DelugeRpcRequest, RencodeValue};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{Mutex, broadcast};
use tokio::task::JoinHandle;

pub struct Connection {
    message_queue: broadcast::WeakSender<DelugeRpcMessage>,
    transport_writer: Arc<Mutex<DelugeWriter>>,
    transport_reader_handle: JoinHandle<()>,
    next_id: AtomicU32,
}

impl Connection {
    pub fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn is_connected(&self) -> bool {
        !self.transport_reader_handle.is_finished()
    }

    pub fn message_queue(&self) -> Option<broadcast::Sender<DelugeRpcMessage>> {
        self.message_queue.upgrade()
    }

    pub async fn send(&self, request: DelugeRpcRequest) -> Result<RencodeValue, DelugeRpcError> {
        let method = request.method.clone();
        let (writer, mut rx) = self.writer_and_rx()?;

        let id = self.next_id();
        {
            let rencode_value = request.into_rencode_value(id);
            let encoded = rencode_value.encode();

            let mut writer = writer.lock().await;
            writer.send(&encoded).await?;
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
                    return Err(DelugeRpcError::RpcError {
                        exc_type,
                        exc_msg,
                        traceback,
                    });
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
                    return Err(DelugeRpcError::ConnectionClosed { method });
                }
            }
        }
    }

    fn writer_and_rx(
        &self,
    ) -> Result<
        (
            Arc<Mutex<DelugeWriter>>,
            broadcast::Receiver<DelugeRpcMessage>,
        ),
        DelugeRpcError,
    > {
        if let Some(message_queue) = self.message_queue.upgrade() {
            return Ok((self.transport_writer.clone(), message_queue.subscribe()));
        }
        Err(DelugeRpcError::NotConnected)
    }

    pub async fn connect(
        host: &str,
        port: u16,
        message_queue_size: usize,
    ) -> Result<Self, TransportError> {
        let transport = DelugeTransport::connect(host, port).await?;
        let (mut transport_reader, transport_writer) = transport.split();

        let writer = Arc::from(Mutex::new(transport_writer));
        let (message_tx, _) = broadcast::channel(message_queue_size);

        let reader_handle = tokio::spawn({
            let message_tx = message_tx.clone();
            async move {
                loop {
                    match transport_reader.recv().await {
                        Ok(raw) => match RencodeValue::decode(&raw) {
                            Ok(decoded) => match DelugeRpcMessage::from_rencode_value(&decoded) {
                                Ok(msg) => {
                                    if let Err(e) = message_tx.send(msg) {
                                        tracing::warn!(error = %e, "failed to send message to response_queue, no subscribers");
                                    }
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
        });

        Ok(Self {
            message_queue: message_tx.downgrade(),
            transport_writer: writer,
            transport_reader_handle: reader_handle,
            next_id: AtomicU32::new(1),
        })
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.transport_reader_handle.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rpc_mock::{Matcher, ReplayServer};
    use tokio::task;

    /// Regression test: when a Connection is dropped, the spawned reader task
    /// must be aborted so it releases its broadcast::Sender (and, transitively,
    /// the TLS read half and its socket FD). Dropping a JoinHandle in tokio is
    /// a no-op, so without an explicit Drop the reader task would leak for the
    /// lifetime of the process.
    #[tokio::test(flavor = "multi_thread")]
    async fn when_connection_dropped_then_reader_task_is_aborted() {
        let server = ReplayServer::start(Matcher::new(vec![]))
            .await
            .expect("start replay server");

        let conn = Connection::connect(&server.host(), server.port(), 16)
            .await
            .expect("connect to replay server");

        // The reader task holds the only strong Sender; the WeakSender on the
        // Connection can upgrade while the task is alive.
        let weak = conn.message_queue.clone();
        assert!(
            weak.upgrade().is_some(),
            "WeakSender should upgrade while reader task is running"
        );

        drop(conn);

        // After Drop aborts the reader task, its Sender is released and the
        // WeakSender can no longer upgrade.
        let mut downgraded = false;
        for _ in 0..10_000 {
            task::yield_now().await;
            if weak.upgrade().is_none() {
                downgraded = true;
                break;
            }
        }
        assert!(
            downgraded,
            "reader task was not aborted on drop; WeakSender still upgrades, \
             meaning the task (and its Sender and socket FD) leaked"
        );

        drop(server);
    }
}
