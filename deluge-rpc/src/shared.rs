use crate::protocol::DelugeRpcMessage;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use tokio::sync::broadcast;

pub struct Shared {
    pub next_id: AtomicU32,
    pub message_tx: broadcast::Sender<DelugeRpcMessage>,
}

impl Shared {
    pub fn new(buffer_capacity: usize) -> Arc<Self> {
        let (message_tx, _) = broadcast::channel(buffer_capacity);
        Arc::new(Self {
            next_id: AtomicU32::new(1),
            message_tx,
        })
    }
}
