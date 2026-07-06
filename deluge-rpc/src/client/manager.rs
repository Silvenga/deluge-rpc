use crate::client::connection::Connection;
use crate::client::info::DelugeConnectionInfo;
use crate::{DelugeRpcError, DelugeRpcRequest, RencodeValue};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ConnectionManager {
    info: Arc<DelugeConnectionInfo>,
    connection: Mutex<Option<Arc<Connection>>>,
}

impl ConnectionManager {
    pub fn new(info: Arc<DelugeConnectionInfo>) -> Self {
        Self {
            info,
            connection: Mutex::new(None),
        }
    }

    pub async fn acquire(&self) -> Result<Arc<Connection>, DelugeRpcError> {
        let mut guard = self.connection.lock().await;

        // Check if the existing connection is valid.
        if let Some(connection) = guard.clone()
            && connection.is_connected()
        {
            return Ok(connection);
        };

        // Connection never existed or is now dead, create a new one.
        let connection: Arc<_> = Connection::connect(
            &self.info.host,
            self.info.port,
            self.info.message_queue_size,
        )
        .await?
        .into();

        connection
            .send(
                DelugeRpcRequest::new("daemon.login")
                    .with_args(vec![
                        RencodeValue::Str(self.info.username.to_owned()),
                        RencodeValue::Str(self.info.password.to_owned()),
                    ])
                    .with_kwargs({
                        let mut kwargs = BTreeMap::new();
                        kwargs.insert(
                            RencodeValue::Str("client_version".into()),
                            RencodeValue::Str(format!("deluge-rpc/{}", env!("CARGO_PKG_VERSION"))),
                        );
                        kwargs
                    }),
            )
            .await?;

        // The old connection will fail until dropped.
        *guard = Some(connection.clone());

        Ok(connection)
    }
}
