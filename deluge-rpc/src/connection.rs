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
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

const BROADCAST_CAPACITY: usize = 256;

pub struct DelugeConnection {
    shared: Arc<Shared>,
    writer: Arc<Mutex<DelugeWriter>>,
    #[expect(dead_code, reason = "owned JoinHandle avoids mem::forget; task continues detached on drop")]
    reader_handle: JoinHandle<()>,
}

impl DelugeConnection {
    pub async fn connect(host: &str, port: u16) -> anyhow::Result<Self> {
        let transport = DelugeTransport::connect(host, port)
            .await
            .context("failed to connect to Deluge daemon")?;
        let (reader, writer) = transport.split();

        let shared = Shared::new(BROADCAST_CAPACITY);

        let reader_handle = tokio::spawn({
            let reader_shared = shared.clone();
            async move {
                reader_loop(reader, reader_shared).await;
            }
        });

        Ok(Self {
            shared,
            writer: Arc::new(Mutex::new(writer)),
            reader_handle,
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

    fn create_client(self) -> DelugeRpcClient {
        DelugeRpcClient::new(self.shared.clone(), self.writer.clone())
    }
}

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
