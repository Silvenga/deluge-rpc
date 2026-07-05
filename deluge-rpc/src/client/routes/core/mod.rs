mod accounts;
mod config;
mod misc;
mod plugins;
mod session;
mod torrents;

pub use accounts::{CoreAccountClient, CoreAccountRpc};
pub use config::{CoreConfigClient, CoreConfigRpc};
pub use misc::{CoreMiscClient, CoreMiscRpc};
pub use plugins::{CorePluginClient, CorePluginRpc};
pub use session::{CoreSessionClient, CoreSessionRpc};
pub use torrents::{CoreTorrentClient, CoreTorrentRpc};

#[cfg(any(test, feature = "mock"))]
pub use accounts::MockCoreAccountRpc;
#[cfg(any(test, feature = "mock"))]
pub use config::MockCoreConfigRpc;
#[cfg(any(test, feature = "mock"))]
pub use misc::MockCoreMiscRpc;
#[cfg(any(test, feature = "mock"))]
pub use plugins::MockCorePluginRpc;
#[cfg(any(test, feature = "mock"))]
pub use session::MockCoreSessionRpc;
#[cfg(any(test, feature = "mock"))]
pub use torrents::MockCoreTorrentRpc;
