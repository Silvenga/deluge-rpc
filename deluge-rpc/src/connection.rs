use crate::DelugeRpcClient;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::{decode_message, extract_single_int};
use crate::rencode::RencodeValue;
use crate::shared::Shared;
use crate::transport::DelugeReader;
use crate::transport::DelugeTransport;
use crate::transport::DelugeWriter;
use anyhow::Context;
use std::collections::BTreeMap;
use std::mem;
use std::sync::Arc;
use tokio::sync::Mutex;

const BROADCAST_CAPACITY: usize = 256;

pub struct DelugeConnection {
    shared: Arc<Shared>,
    writer: Arc<Mutex<DelugeWriter>>,
}

impl DelugeConnection {
    pub async fn connect(host: &str, port: u16) -> anyhow::Result<Self> {
        let transport = DelugeTransport::connect(host, port)
            .await
            .context("failed to connect to Deluge daemon")?;
        let (mut reader, writer) = transport.split();

        let shared = Shared::new(BROADCAST_CAPACITY);
        let reader_shared = Arc::clone(&shared);

        // Detach the reader task: it runs for the connection's lifetime and
        // exits naturally when the socket closes (recv returns EOF). We
        // intentionally leak the JoinHandle - DelugeConnection is consumed by
        // login()/create_client(), and a Drop impl that aborts the task would
        // kill the reader before any RPC call can complete. The task holds no
        // resources beyond the ReadHalf, which is dropped when recv() errors.
        let _reader_task = tokio::spawn(async move {
            reader_loop(&mut reader, &reader_shared).await;
        });
        mem::forget(_reader_task);

        Ok(Self {
            shared,
            writer: Arc::new(Mutex::new(writer)),
        })
    }

    pub async fn login(self, username: &str, password: &str) -> anyhow::Result<DelugeRpcClient> {
        let mut kwargs = BTreeMap::new();
        kwargs.insert(
            RencodeValue::Str(String::from("client_version")),
            RencodeValue::Str(String::from(concat!(
                "deluge-rpc/",
                env!("CARGO_PKG_VERSION")
            ))),
        );

        let args = vec![
            RencodeValue::Str(username.to_owned()),
            RencodeValue::Str(password.to_owned()),
        ];

        let request = DelugeRpcRequest::new("daemon.login")
            .with_args(args)
            .with_kwargs(kwargs);

        let client = self.create_client();
        let result = client
            .rpc_call(request)
            .await
            .context("daemon.login RPC failed")?;

        let auth_level = extract_single_int(&result, "daemon.login")?;
        tracing::debug!(auth_level, "daemon.login succeeded");
        Ok(client)
    }

    pub fn create_client(self) -> DelugeRpcClient {
        DelugeRpcClient::new(Arc::clone(&self.shared), Arc::clone(&self.writer))
    }
}

async fn reader_loop(reader: &mut DelugeReader, shared: &Arc<Shared>) {
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
