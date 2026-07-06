use crate::{DelugeRpcMessage, DelugeTransport, DelugeWriter, RencodeValue};
use anyhow::{bail, Context};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;

const BROADCAST_CAPACITY: usize = 256;

pub struct Connection {
    message_queue: broadcast::WeakSender<DelugeRpcMessage>,
    transport_writer: Arc<Mutex<DelugeWriter>>,
    transport_reader_handle: JoinHandle<()>,
    next_id: AtomicU32,
}

impl Connection {
    pub async fn connect(host: &str, port: u16) -> anyhow::Result<Self> {
        let transport = DelugeTransport::connect(host, port)
            .await
            .context("failed to connect to Deluge daemon")?;
        let (mut transport_reader, transport_writer) = transport.split();

        let writer = Arc::from(Mutex::new(transport_writer));
        let (message_tx, _) = broadcast::channel(BROADCAST_CAPACITY);

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

    pub fn writer_and_rx(
        &self,
    ) -> anyhow::Result<(
        Arc<Mutex<DelugeWriter>>,
        broadcast::Receiver<DelugeRpcMessage>,
    )> {
        if let Some(message_queue) = self.message_queue.upgrade() {
            return Ok((self.transport_writer.clone(), message_queue.subscribe()));
        }
        bail!("Connection already closed")
    }

    pub fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn is_connected(&self) -> bool {
        !self.transport_reader_handle.is_finished()
    }
}
