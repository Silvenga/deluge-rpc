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

#[cfg(feature = "mock")]
pub use accounts::MockCoreAccountRpc;
#[cfg(feature = "mock")]
pub use config::MockCoreConfigRpc;
#[cfg(feature = "mock")]
pub use misc::MockCoreMiscRpc;
#[cfg(feature = "mock")]
pub use plugins::MockCorePluginRpc;
#[cfg(feature = "mock")]
pub use session::MockCoreSessionRpc;
#[cfg(feature = "mock")]
pub use torrents::MockCoreTorrentRpc;
