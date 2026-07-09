#[cfg(feature = "recorder")]
use crate::recorder::RecordedInteraction;
use std::time::Duration;
#[cfg(feature = "recorder")]
use tokio::sync::mpsc;

/// Connection parameters for a Deluge daemon.
pub(crate) struct DelugeConnectionInfo {
    /// The hostname or IP address of the Deluge daemon.
    pub host: String,
    /// The TCP port of the Deluge daemon.
    pub port: u16,
    /// The username for authentication.
    pub username: String,
    /// The password for authentication.
    pub password: String,
    /// The timeout for an RPC request. This includes
    /// connecting (if needed)/login (if needed)/sending the request, and waiting for the response.
    /// Each rpc call is multiplexed on the same connection. Each call has an independent timeout.
    pub rpc_timeout: Duration,
    /// The maximum numbers of messages received that will be buffered before being dropped.
    pub message_queue_size: usize,
    /// The maximum numbers of events received that will be buffered before being dropped.
    pub event_queue_size: usize,
    /// Channel to send recorded request-response interactions to (requires `recorder` feature).
    #[cfg(feature = "recorder")]
    pub recorder_tx: Option<mpsc::Sender<RecordedInteraction>>,
}
