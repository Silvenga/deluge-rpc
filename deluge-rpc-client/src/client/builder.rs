use crate::DelugeClient;
use crate::client::info::DelugeConnectionInfo;
#[cfg(feature = "recorder")]
use crate::recorder::RecordedInteraction;
use std::time::Duration;
#[cfg(feature = "recorder")]
use tokio::sync::mpsc;

const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_MESSAGE_QUEUE_SIZE: usize = 256;
const DEFAULT_EVENT_QUEUE_SIZE: usize = 256;

/// Builder for building a [crate::DelugeClient].
pub struct DelugeClientBuilder {
    host: String,
    port: u16,
    username: String,
    password: String,
    rpc_timeout: Duration,
    message_queue_size: usize,
    event_queue_size: usize,
    #[cfg(feature = "recorder")]
    recorder_tx: Option<mpsc::Sender<RecordedInteraction>>,
}

impl DelugeClientBuilder {
    /// Create a new builder with the given connection credentials.
    pub fn new(
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            username: username.into(),
            password: password.into(),
            rpc_timeout: DEFAULT_RPC_TIMEOUT,
            message_queue_size: MAX_MESSAGE_QUEUE_SIZE,
            event_queue_size: DEFAULT_EVENT_QUEUE_SIZE,
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

    /// Sets the maximum number of events to buffer in the event stream channel.
    /// Defaults to 256.
    pub fn with_event_queue_size(mut self, size: usize) -> Self {
        self.event_queue_size = size;
        self
    }

    /// Enable recording of request-response interactions (requires `recorder` feature).
    #[cfg(feature = "recorder")]
    pub fn with_recorder(mut self, tx: mpsc::Sender<RecordedInteraction>) -> Self {
        self.recorder_tx = Some(tx);
        self
    }

    /// Build the `DelugeClient` with the configured parameters.
    pub fn build(self) -> DelugeClient {
        DelugeClient::new(DelugeConnectionInfo {
            host: self.host,
            port: self.port,
            username: self.username,
            password: self.password,
            rpc_timeout: self.rpc_timeout,
            message_queue_size: self.message_queue_size,
            event_queue_size: self.event_queue_size,
            #[cfg(feature = "recorder")]
            recorder_tx: self.recorder_tx,
        })
    }
}
