mod auto_add;
mod blocklist;
mod execute;
mod extractor;
mod label;
mod notifications;
mod scheduler;
mod stats;
mod toggle;
mod webui;

pub use auto_add::{AutoAddClient, AutoAddRpc};
pub use blocklist::{BlocklistClient, BlocklistRpc};
pub use execute::{ExecuteClient, ExecuteRpc};
pub use extractor::{ExtractorClient, ExtractorRpc};
pub use label::{LabelClient, LabelRpc};
pub use notifications::{NotificationsClient, NotificationsRpc};
pub use scheduler::{SchedulerClient, SchedulerRpc};
pub use stats::{StatsClient, StatsRpc};
pub use toggle::{ToggleClient, ToggleRpc};
pub use webui::{WebUiClient, WebUiRpc};

#[cfg(feature = "mock")]
pub use auto_add::MockAutoAddRpc;
#[cfg(feature = "mock")]
pub use blocklist::MockBlocklistRpc;
#[cfg(feature = "mock")]
pub use execute::MockExecuteRpc;
#[cfg(feature = "mock")]
pub use extractor::MockExtractorRpc;
#[cfg(feature = "mock")]
pub use label::MockLabelRpc;
#[cfg(feature = "mock")]
pub use notifications::MockNotificationsRpc;
#[cfg(feature = "mock")]
pub use scheduler::MockSchedulerRpc;
#[cfg(feature = "mock")]
pub use stats::MockStatsRpc;
#[cfg(feature = "mock")]
pub use toggle::MockToggleRpc;
#[cfg(feature = "mock")]
pub use webui::MockWebUiRpc;
