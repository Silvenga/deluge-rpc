use crate::DelugeClient;
use crate::client::info::DelugeConnectionInfo;
#[cfg(feature = "recorder")]
use crate::recorder::RecordedInteraction;
use std::time::Duration;
#[cfg(feature = "recorder")]
use tokio::sync::mpsc;

const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_MESSAGE_QUEUE_SIZE: usize = 256;

pub struct DelugeClientBuilder {
    host: String,
    port: u16,
    username: String,
    password: String,
    rpc_timeout: Duration,
    message_queue_size: usize,
    #[cfg(feature = "recorder")]
    recorder_tx: Option<mpsc::Sender<RecordedInteraction>>,
}

impl DelugeClientBuilder {
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
            rpc_timeout: DEFAULT_RPC_TIMEOUT,
            message_queue_size: MAX_MESSAGE_QUEUE_SIZE,
            #[cfg(feature = "recorder")]
            recorder_tx: None,
        }
    }

    /// Sets the RPC timeout duration for the instance.
    /// Defaults to 30s.
    pub fn with_rpc_timeout(mut self, timeout: Duration) -> Self {
        self.rpc_timeout = timeout;
        self
    }

    /// Sets the maximum of messages (responses or events) to buffer.
    /// Defaults to 256.
    pub fn with_message_queue_size(mut self, size: usize) -> Self {
        self.message_queue_size = size;
        self
    }

    #[cfg(feature = "recorder")]
    pub fn with_recorder(mut self, tx: mpsc::Sender<RecordedInteraction>) -> Self {
        self.recorder_tx = Some(tx);
        self
    }

    pub fn build(self) -> DelugeClient {
        DelugeClient::new(DelugeConnectionInfo {
            host: self.host,
            port: self.port,
            username: self.username,
            password: self.password,
            rpc_timeout: self.rpc_timeout,
            message_queue_size: self.message_queue_size,
            #[cfg(feature = "recorder")]
            recorder_tx: self.recorder_tx,
        })
    }
}
