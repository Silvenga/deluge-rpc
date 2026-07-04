mod client;
mod connection;
pub mod models;
mod protocol;
mod rencode;
mod shared;
mod transport;

pub use client::RpcCaller;
pub use client::core::{
    CoreAccountClient, CoreAccountRpc, CoreConfigClient, CoreConfigRpc, CoreMiscClient,
    CoreMiscRpc, CorePluginClient, CorePluginRpc, CoreSessionClient, CoreSessionRpc,
    CoreTorrentClient, CoreTorrentRpc,
};
pub use client::daemon::{DaemonClient, DaemonRpc};
pub use client::deluge_client::{CoreClient, DelugeClient, PluginsClient};
pub use client::plugins::{LabelClient, LabelRpc};
pub use protocol::{
    DelugeRpcMessage, DelugeRpcRequest, extract_single, extract_single_dict, extract_single_int,
};
pub use rencode::RencodeValue;
pub use rencode::from_json;
pub use rencode::to_json;
pub use rencode::to_rencode_value;
pub use transport::TransportError;

#[cfg(any(test, feature = "mock"))]
pub use client::core::{
    MockCoreAccountRpc, MockCoreConfigRpc, MockCoreMiscRpc, MockCorePluginRpc, MockCoreSessionRpc,
    MockCoreTorrentRpc,
};
#[cfg(any(test, feature = "mock"))]
pub use client::daemon::MockDaemonRpc;
