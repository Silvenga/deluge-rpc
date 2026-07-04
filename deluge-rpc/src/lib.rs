mod client;
mod connection;
pub mod models;
mod protocol;
mod rencode;
mod rpc;
mod shared;
mod transport;

pub use client::DelugeRpcClient;
pub use client::core::{
    CoreAccountClient, CoreAccountRpc, CoreConfigClient, CoreConfigRpc, CoreMiscClient,
    CoreMiscRpc, CorePluginClient, CorePluginRpc, CoreSessionClient, CoreSessionRpc,
    CoreTorrentClient, CoreTorrentRpc,
};
pub use client::daemon::{DaemonClient, DaemonRpc};
pub use client::deluge_client::{CoreClient, DelugeClient, PluginsClient};
pub use connection::DelugeConnection;
pub use protocol::{DelugeRpcMessage, DelugeRpcRequest};
pub use rencode::RencodeValue;
pub use rpc::DelugeRpc;
pub use transport::TransportError;

#[cfg(any(test, feature = "mock"))]
pub use client::core::{
    MockCoreAccountRpc, MockCoreConfigRpc, MockCoreMiscRpc, MockCorePluginRpc,
    MockCoreSessionRpc, MockCoreTorrentRpc,
};
#[cfg(any(test, feature = "mock"))]
pub use client::daemon::MockDaemonRpc;
#[cfg(any(test, feature = "mock"))]
pub use rpc::MockDelugeRpc;
