use crate::client::shared::Shared;
use crate::{
    DelugeReader, DelugeRpcMessage, DelugeRpcRequest, DelugeTransport, DelugeWriter, RencodeValue,
};
use anyhow::{anyhow, Context};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio::time::timeout;

const BROADCAST_CAPACITY: usize = 256;
const RPC_TIMEOUT: Duration = Duration::from_secs(30);

pub enum ConnectionState {
    Connected {
        shared: Arc<Shared>,
        writer: Arc<Mutex<DelugeWriter>>,
        reader_handle: JoinHandle<()>,
    },
    Disconnected,
}

impl ConnectionState {
    fn is_connected(&self) -> bool {
        match self {
            ConnectionState::Connected { reader_handle, .. } => !reader_handle.is_finished(),
            ConnectionState::Disconnected => false,
        }
    }

    pub(crate) async fn ensure_connected(
        &mut self,
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> anyhow::Result<()> {
        if self.is_connected() {
            return Ok(());
        }

        let transport = DelugeTransport::connect(host, port)
            .await
            .context("failed to connect to Deluge daemon")?;
        let (reader, writer) = transport.split();

        let shared = Shared::new(BROADCAST_CAPACITY);
        let writer = Arc::new(Mutex::new(writer));

        let reader_shared = shared.clone();
        let reader_handle = tokio::spawn(async move {
            reader_loop(reader, reader_shared).await;
        });

        let login_result = {
            let login_id = shared.next_id.fetch_add(1, Ordering::Relaxed);

            let mut kwargs = BTreeMap::new();
            kwargs.insert(
                RencodeValue::Str("client_version".into()),
                RencodeValue::Str(format!("deluge-rpc/{}", env!("CARGO_PKG_VERSION"))),
            );

            let login_request = DelugeRpcRequest::new("daemon.login")
                .with_args(vec![
                    RencodeValue::Str(username.to_owned()),
                    RencodeValue::Str(password.to_owned()),
                ])
                .with_kwargs(kwargs);
            let login_encoded = login_request.encode(login_id);

            let mut rx = shared.message_tx.subscribe();

            {
                let mut w = writer.lock().await;
                w.send(&login_encoded)
                    .await
                    .context("failed to send daemon.login")?;
            }

            timeout(RPC_TIMEOUT, async {
                loop {
                    match rx.recv().await {
                        Ok(DelugeRpcMessage::Response { id, .. }) if id == login_id => {
                            return Ok::<_, anyhow::Error>(());
                        }
                        Ok(DelugeRpcMessage::Error {
                            id,
                            exc_type,
                            exc_msg,
                            ..
                        }) if id == login_id => {
                            return Err(anyhow!("daemon.login failed: {exc_type}: {exc_msg}"));
                        }
                        Ok(_) => continue,
                        Err(RecvError::Lagged(n)) => {
                            tracing::warn!(n, "login subscriber lagged");
                            continue;
                        }
                        Err(RecvError::Closed) => {
                            return Err(anyhow!("connection closed during daemon.login"));
                        }
                    }
                }
            })
            .await
        };

        match login_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                reader_handle.abort();
                return Err(e);
            }
            Err(_) => {
                reader_handle.abort();
                return Err(anyhow!("timed out waiting for daemon.login response"));
            }
        }

        *self = ConnectionState::Connected {
            shared,
            writer,
            reader_handle,
        };

        Ok(())
    }

    pub(crate) fn writer_and_rx(
        &self,
    ) -> Option<(
        Arc<Mutex<DelugeWriter>>,
        broadcast::Receiver<DelugeRpcMessage>,
    )> {
        match self {
            ConnectionState::Connected { writer, shared, .. } => {
                let rx = shared.message_tx.subscribe();
                Some((Arc::clone(writer), rx))
            }
            ConnectionState::Disconnected => None,
        }
    }
}

async fn reader_loop(mut reader: DelugeReader, shared: Arc<Shared>) {
    loop {
        match reader.recv().await {
            Ok(raw) => match RencodeValue::decode(&raw) {
                Ok(decoded) => match DelugeRpcMessage::from_rencode_value(&decoded) {
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
