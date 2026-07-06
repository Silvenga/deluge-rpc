use std::time::Duration;

pub struct DelugeConnectionInfo {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    /// The timeout for an RPC request. This includes
    /// connecting (if needed)/login (if needed)/sending the request, and waiting for the response.
    /// Each rpc call is multiplexed on the same connection. Each call has an independent timeout.
    pub rpc_timeout: Duration,
    /// The maximum numbers of messages received that will be buffered.
    pub message_queue_size: usize,
}
