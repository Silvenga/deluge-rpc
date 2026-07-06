use crate::client::connection::Connection;
use crate::client::info::DelugeConnectionInfo;
use crate::client::dispatch::dispatch;
use crate::{DelugeRpcRequest, RencodeValue};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ConnectionManager {
    connection_info: DelugeConnectionInfo,
    connection: Mutex<Option<Arc<Connection>>>,
}

impl ConnectionManager {
    pub fn new(connection_info: DelugeConnectionInfo) -> Self {
        Self {
            connection_info,
            connection: Mutex::new(None),
        }
    }

    pub async fn acquire(&self) -> anyhow::Result<Arc<Connection>> {
        let mut guard = self.connection.lock().await;

        if let Some(connection) = guard.clone()
            && connection.is_connected()
        {
            return Ok(connection);
        };

        // Connection never existed or is now dead.
        let connection: Arc<_> =
            Connection::connect(&self.connection_info.host, self.connection_info.port)
                .await?
                .into();

        let login_request = DelugeRpcRequest::new("daemon.login")
            .with_args(vec![
                RencodeValue::Str(self.connection_info.username.to_owned()),
                RencodeValue::Str(self.connection_info.password.to_owned()),
            ])
            .with_kwargs({
                let mut kwargs = BTreeMap::new();
                kwargs.insert(
                    RencodeValue::Str("client_version".into()),
                    RencodeValue::Str(format!("deluge-rpc/{}", env!("CARGO_PKG_VERSION"))),
                );
                kwargs
            });

        dispatch(&connection, login_request).await?;

        *guard = Some(connection.clone());

        Ok(connection)
    }
}
