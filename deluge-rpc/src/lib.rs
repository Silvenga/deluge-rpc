mod client;
mod connection;
pub mod models;
mod protocol;
mod rencode;
mod shared;
mod transport;

pub use client::core::{
    CoreAccountClient, CoreAccountRpc, CoreConfigClient, CoreConfigRpc, CoreMiscClient,
    CoreMiscRpc, CorePluginClient, CorePluginRpc, CoreSessionClient, CoreSessionRpc,
    CoreTorrentClient, CoreTorrentRpc,
};
pub use client::daemon::{DaemonClient, DaemonRpc};
pub use client::deluge_client::{CoreClient, DelugeClient, PluginsClient};
pub use protocol::{DelugeRpcMessage, DelugeRpcRequest};
pub use rencode::RencodeValue;
pub use transport::TransportError;

#[cfg(any(test, feature = "mock"))]
pub use client::core::{
    MockCoreAccountRpc, MockCoreConfigRpc, MockCoreMiscRpc, MockCorePluginRpc,
    MockCoreSessionRpc, MockCoreTorrentRpc,
};
#[cfg(any(test, feature = "mock"))]
pub use client::daemon::MockDaemonRpc;
