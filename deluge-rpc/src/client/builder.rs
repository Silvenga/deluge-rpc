use crate::DelugeClient;
use crate::client::info::DelugeConnectionInfo;
use std::time::Duration;

const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_MESSAGE_QUEUE_SIZE: usize = 256;

pub struct DelugeClientBuilder {
    host: String,
    port: u16,
    username: String,
    password: String,
    rpc_timeout: Duration,
    message_queue_size: usize,
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

    pub fn build(self) -> DelugeClient {
        DelugeClient::new(DelugeConnectionInfo {
            host: self.host,
            port: self.port,
            username: self.username,
            password: self.password,
            rpc_timeout: self.rpc_timeout,
            message_queue_size: self.message_queue_size,
        })
    }
}
