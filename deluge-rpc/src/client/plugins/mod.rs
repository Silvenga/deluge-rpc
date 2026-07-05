mod autoadd;
mod blocklist;
mod execute;
mod extractor;
mod label;
mod notifications;
mod scheduler;
mod stats;
mod toggle;
mod webui;

pub use autoadd::{AutoaddClient, AutoaddRpc};
pub use blocklist::{BlocklistClient, BlocklistRpc};
pub use execute::{ExecuteClient, ExecuteRpc};
pub use extractor::{ExtractorClient, ExtractorRpc};
pub use label::{LabelClient, LabelRpc};
pub use notifications::{NotificationsClient, NotificationsRpc};
pub use scheduler::{SchedulerClient, SchedulerRpc};
pub use stats::{StatsClient, StatsRpc};
pub use toggle::{ToggleClient, ToggleRpc};
pub use webui::{WebuiClient, WebuiRpc};

#[cfg(any(test, feature = "mock"))]
pub use autoadd::MockAutoaddRpc;
#[cfg(any(test, feature = "mock"))]
pub use blocklist::MockBlocklistRpc;
#[cfg(any(test, feature = "mock"))]
pub use execute::MockExecuteRpc;
#[cfg(any(test, feature = "mock"))]
pub use extractor::MockExtractorRpc;
#[cfg(any(test, feature = "mock"))]
pub use label::MockLabelRpc;
#[cfg(any(test, feature = "mock"))]
pub use notifications::MockNotificationsRpc;
#[cfg(any(test, feature = "mock"))]
pub use scheduler::MockSchedulerRpc;
#[cfg(any(test, feature = "mock"))]
pub use stats::MockStatsRpc;
#[cfg(any(test, feature = "mock"))]
pub use toggle::MockToggleRpc;
#[cfg(any(test, feature = "mock"))]
pub use webui::MockWebuiRpc;
