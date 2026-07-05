use crate::client::connection::ConnectionState;
use std::sync::atomic::AtomicU32;
use tokio::sync::Mutex;

pub struct DelugeClientInner {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub next_id: AtomicU32,
    pub state: Mutex<ConnectionState>,
}

impl DelugeClientInner {
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
            next_id: AtomicU32::new(1),
            state: Mutex::new(ConnectionState::Disconnected),
        }
    }
}
