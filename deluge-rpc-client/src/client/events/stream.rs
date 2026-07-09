use crate::client::connection::Connection;
use crate::client::events::reader_loop::reader_loop;
use crate::{DelugeRpcError, DelugeRpcRequest};
use deluge_rpc_models::DelugeEvent;
use deluge_rpc_rencode::RencodeValue;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_stream::Stream;

/// An async stream of [`DelugeEvent`]s from a dedicated daemon connection.
///
/// The underlying connection is closed when this stream is dropped.
/// If the connection dies, the stream yields an error and then ends.
pub struct EventStream {
    rx: mpsc::Receiver<Result<DelugeEvent, DelugeRpcError>>,
    shutdown: Option<Arc<ShutdownHandle>>,
}

impl EventStream {
    /// Creates a dedicated connection, subscribes to the given event names, and
    /// returns a stream of events. The connection is closed when the stream is dropped.
    pub(crate) async fn subscribe(
        connection: Connection,
        event_names: &[String],
        channel_capacity: usize,
    ) -> Result<Self, DelugeRpcError> {
        let names: Vec<RencodeValue> = event_names
            .iter()
            .map(|n| RencodeValue::Str(n.clone()))
            .collect();
        let connection = Arc::from(connection);

        connection
            .send(
                DelugeRpcRequest::new("daemon.set_event_interest")
                    .with_args(vec![RencodeValue::List(names)]),
            )
            .await?;

        let (tx, rx) = mpsc::channel(channel_capacity);

        let reader_handle = tokio::spawn(reader_loop(connection.clone(), tx));

        let shutdown = Arc::new(ShutdownHandle {
            reader_handle,
            _connection: connection,
        });

        Ok(Self {
            rx,
            shutdown: Some(shutdown),
        })
    }
}

impl Drop for EventStream {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            shutdown.reader_handle.abort();
        }
    }
}

impl Stream for EventStream {
    type Item = Result<DelugeEvent, DelugeRpcError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

struct ShutdownHandle {
    reader_handle: JoinHandle<()>,
    _connection: Arc<Connection>,
}
