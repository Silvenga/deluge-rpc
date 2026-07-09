use clap::Subcommand;
use deluge_rpc_client::DelugeClient;
use deluge_rpc_client::models::{
    AutoAddConfig, BlocklistConfig, ExecuteEvent, ExtractorConfig, LabelConfig, LabelOptions,
    NotificationsConfig, SchedulerConfig, StatsConfig, WatchDirOptions, WebUiConfig,
};

/// Plugin RPC methods. Plugin methods only exist when the plugin is enabled.
#[derive(Subcommand, Debug, Clone)]
pub enum PluginsCommand {
    #[command(subcommand)]
    Label(LabelCommand),
    #[command(subcommand)]
    AutoAdd(AutoAddCommand),
    #[command(subcommand)]
    Blocklist(BlocklistCommand),
    #[command(subcommand)]
    Execute(ExecuteCommand),
    #[command(subcommand)]
    Extractor(ExtractorCommand),
    #[command(subcommand)]
    Notifications(NotificationsCommand),
    #[command(subcommand)]
    Scheduler(SchedulerCommand),
    #[command(subcommand)]
    Stats(StatsCommand),
    #[command(subcommand)]
    Toggle(ToggleCommand),
    #[command(subcommand)]
    WebUi(WebUiCommand),
}

impl PluginsCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            PluginsCommand::Label(cmd) => cmd.run(client).await,
            PluginsCommand::AutoAdd(cmd) => cmd.run(client).await,
            PluginsCommand::Blocklist(cmd) => cmd.run(client).await,
            PluginsCommand::Execute(cmd) => cmd.run(client).await,
            PluginsCommand::Extractor(cmd) => cmd.run(client).await,
            PluginsCommand::Notifications(cmd) => cmd.run(client).await,
            PluginsCommand::Scheduler(cmd) => cmd.run(client).await,
            PluginsCommand::Stats(cmd) => cmd.run(client).await,
            PluginsCommand::Toggle(cmd) => cmd.run(client).await,
            PluginsCommand::WebUi(cmd) => cmd.run(client).await,
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum LabelCommand {
    #[command(name = "list")]
    List,
    #[command(name = "add")]
    Add { id: String },
    #[command(name = "remove")]
    Remove { id: String },
    #[command(name = "set-options")]
    SetOptions { id: String, json: String },
    #[command(name = "get-options")]
    GetOptions { id: String },
    #[command(name = "set-torrent")]
    SetTorrent {
        torrent_id: String,
        label_id: String,
    },
    #[command(name = "get-config")]
    GetConfig,
    #[command(name = "set-config")]
    SetConfig { json: String },
}

impl LabelCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            LabelCommand::List => {
                let labels = client.plugins.label.get_labels().await?;
                Ok(serde_json::to_string_pretty(&labels)?)
            }
            LabelCommand::Add { id } => {
                client.plugins.label.add(id).await?;
                Ok(format!("Label '{id}' added."))
            }
            LabelCommand::Remove { id } => {
                client.plugins.label.remove(id).await?;
                Ok(format!("Label '{id}' removed."))
            }
            LabelCommand::SetOptions { id, json } => {
                let options: LabelOptions = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse options: {e}"))?;
                client.plugins.label.set_options(id, &options).await?;
                Ok("Options set.".to_owned())
            }
            LabelCommand::GetOptions { id } => {
                let options = client.plugins.label.get_options(id).await?;
                Ok(serde_json::to_string_pretty(&options)?)
            }
            LabelCommand::SetTorrent {
                torrent_id,
                label_id,
            } => {
                client
                    .plugins
                    .label
                    .set_torrent(torrent_id, label_id)
                    .await?;
                Ok("Label set on torrent.".to_owned())
            }
            LabelCommand::GetConfig => {
                let config = client.plugins.label.get_config().await?;
                Ok(serde_json::to_string_pretty(&config)?)
            }
            LabelCommand::SetConfig { json } => {
                let config: LabelConfig = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;
                client.plugins.label.set_config(&config).await?;
                Ok("Config set.".to_owned())
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum AutoAddCommand {
    #[command(name = "get-config")]
    GetConfig,
    #[command(name = "set-config")]
    SetConfig { json: String },
    #[command(name = "get-watchdirs")]
    GetWatchdirs,
    #[command(name = "add")]
    Add { json: Option<String> },
    #[command(name = "remove")]
    Remove { id: i64 },
    #[command(name = "enable-watchdir")]
    EnableWatchdir { id: i64 },
    #[command(name = "disable-watchdir")]
    DisableWatchdir { id: i64 },
    #[command(name = "set-options")]
    SetOptions { id: i64, json: String },
    #[command(name = "is-admin")]
    IsAdmin,
    #[command(name = "get-auth-user")]
    GetAuthUser,
}

impl AutoAddCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            AutoAddCommand::GetConfig => {
                let config = client.plugins.auto_add.get_config().await?;
                Ok(serde_json::to_string_pretty(&config)?)
            }
            AutoAddCommand::SetConfig { json } => {
                let config: AutoAddConfig = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;
                client.plugins.auto_add.set_config(&config).await?;
                Ok("Config set.".to_owned())
            }
            AutoAddCommand::GetWatchdirs => {
                let dirs = client.plugins.auto_add.get_watch_dirs().await?;
                Ok(serde_json::to_string_pretty(&dirs)?)
            }
            AutoAddCommand::Add { json } => {
                let options = match json {
                    Some(j) => Some(
                        serde_json::from_str::<WatchDirOptions>(j)
                            .map_err(|e| anyhow::anyhow!("failed to parse options: {e}"))?,
                    ),
                    None => None,
                };
                let id = client.plugins.auto_add.add(options).await?;
                Ok(serde_json::to_string_pretty(&id)?)
            }
            AutoAddCommand::Remove { id } => {
                client.plugins.auto_add.remove(*id).await?;
                Ok("Watchdir removed.".to_owned())
            }
            AutoAddCommand::EnableWatchdir { id } => {
                client.plugins.auto_add.enable_watch_dir(*id).await?;
                Ok("Watchdir enabled.".to_owned())
            }
            AutoAddCommand::DisableWatchdir { id } => {
                client.plugins.auto_add.disable_watch_dir(*id).await?;
                Ok("Watchdir disabled.".to_owned())
            }
            AutoAddCommand::SetOptions { id, json } => {
                let options: WatchDirOptions = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse options: {e}"))?;
                client.plugins.auto_add.set_options(*id, &options).await?;
                Ok("Options set.".to_owned())
            }
            AutoAddCommand::IsAdmin => {
                let result = client.plugins.auto_add.is_admin_level().await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            AutoAddCommand::GetAuthUser => {
                let result = client.plugins.auto_add.get_auth_user().await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum BlocklistCommand {
    #[command(name = "check-import")]
    CheckImport {
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    #[command(name = "get-config")]
    GetConfig,
    #[command(name = "set-config")]
    SetConfig { json: String },
    #[command(name = "get-status")]
    GetStatus,
}

impl BlocklistCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            BlocklistCommand::CheckImport { force } => {
                let result = client.plugins.blocklist.check_import(*force).await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            BlocklistCommand::GetConfig => {
                let config = client.plugins.blocklist.get_config().await?;
                Ok(serde_json::to_string_pretty(&config)?)
            }
            BlocklistCommand::SetConfig { json } => {
                let config: BlocklistConfig = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;
                client.plugins.blocklist.set_config(&config).await?;
                Ok("Config set.".to_owned())
            }
            BlocklistCommand::GetStatus => {
                let status = client.plugins.blocklist.get_status().await?;
                Ok(serde_json::to_string_pretty(&status)?)
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum ExecuteCommand {
    #[command(name = "add-command")]
    AddCommand { event: String, command: String },
    #[command(name = "get-commands")]
    GetCommands,
    #[command(name = "remove-command")]
    RemoveCommand { id: String },
    #[command(name = "save-command")]
    SaveCommand {
        id: String,
        event: String,
        command: String,
    },
}

impl ExecuteCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            ExecuteCommand::AddCommand { event, command } => {
                let evt = parse_execute_event(event)?;
                client.plugins.execute.add_command(&evt, command).await?;
                Ok("Command added.".to_owned())
            }
            ExecuteCommand::GetCommands => {
                let commands = client.plugins.execute.get_commands().await?;
                Ok(serde_json::to_string_pretty(&commands)?)
            }
            ExecuteCommand::RemoveCommand { id } => {
                client.plugins.execute.remove_command(id).await?;
                Ok("Command removed.".to_owned())
            }
            ExecuteCommand::SaveCommand { id, event, command } => {
                let evt = parse_execute_event(event)?;
                client
                    .plugins
                    .execute
                    .save_command(id, &evt, command)
                    .await?;
                Ok("Command saved.".to_owned())
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum ExtractorCommand {
    #[command(name = "get-config")]
    GetConfig,
    #[command(name = "set-config")]
    SetConfig { json: String },
}

impl ExtractorCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            ExtractorCommand::GetConfig => {
                let config = client.plugins.extractor.get_config().await?;
                Ok(serde_json::to_string_pretty(&config)?)
            }
            ExtractorCommand::SetConfig { json } => {
                let config: ExtractorConfig = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;
                client.plugins.extractor.set_config(&config).await?;
                Ok("Config set.".to_owned())
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum NotificationsCommand {
    #[command(name = "get-config")]
    GetConfig,
    #[command(name = "set-config")]
    SetConfig { json: String },
    #[command(name = "get-handled-events")]
    GetHandledEvents,
}

impl NotificationsCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            NotificationsCommand::GetConfig => {
                let config = client.plugins.notifications.get_config().await?;
                Ok(serde_json::to_string_pretty(&config)?)
            }
            NotificationsCommand::SetConfig { json } => {
                let config: NotificationsConfig = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;
                client.plugins.notifications.set_config(&config).await?;
                Ok("Config set.".to_owned())
            }
            NotificationsCommand::GetHandledEvents => {
                let events = client.plugins.notifications.get_handled_events().await?;
                Ok(serde_json::to_string_pretty(&events)?)
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum SchedulerCommand {
    #[command(name = "get-config")]
    GetConfig,
    #[command(name = "set-config")]
    SetConfig { json: String },
    #[command(name = "get-state")]
    GetState,
}

impl SchedulerCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            SchedulerCommand::GetConfig => {
                let config = client.plugins.scheduler.get_config().await?;
                Ok(serde_json::to_string_pretty(&config)?)
            }
            SchedulerCommand::SetConfig { json } => {
                let config: SchedulerConfig = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;
                client.plugins.scheduler.set_config(&config).await?;
                Ok("Config set.".to_owned())
            }
            SchedulerCommand::GetState => {
                let state = client.plugins.scheduler.get_state().await?;
                Ok(serde_json::to_string_pretty(&state)?)
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum StatsCommand {
    #[command(name = "get-stats")]
    GetStats { keys: String, interval: i64 },
    #[command(name = "get-totals")]
    GetTotals,
    #[command(name = "get-session-totals")]
    GetSessionTotals,
    #[command(name = "get-config")]
    GetConfig,
    #[command(name = "set-config")]
    SetConfig { json: String },
    #[command(name = "get-intervals")]
    GetIntervals,
}

impl StatsCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            StatsCommand::GetStats { keys, interval } => {
                let keys_list: Vec<String> = serde_json::from_str(keys)
                    .map_err(|e| anyhow::anyhow!("failed to parse keys: {e}"))?;
                let result = client
                    .plugins
                    .stats
                    .get_stats(&keys_list, *interval)
                    .await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            StatsCommand::GetTotals => {
                let totals = client.plugins.stats.get_totals().await?;
                Ok(serde_json::to_string_pretty(&totals)?)
            }
            StatsCommand::GetSessionTotals => {
                let totals = client.plugins.stats.get_session_totals().await?;
                Ok(serde_json::to_string_pretty(&totals)?)
            }
            StatsCommand::GetConfig => {
                let config = client.plugins.stats.get_config().await?;
                Ok(serde_json::to_string_pretty(&config)?)
            }
            StatsCommand::SetConfig { json } => {
                let config: StatsConfig = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;
                client.plugins.stats.set_config(&config).await?;
                Ok("Config set.".to_owned())
            }
            StatsCommand::GetIntervals => {
                let intervals = client.plugins.stats.get_intervals().await?;
                Ok(serde_json::to_string_pretty(&intervals)?)
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum ToggleCommand {
    #[command(name = "get-status")]
    GetStatus,
    #[command(name = "toggle")]
    Toggle,
}

impl ToggleCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            ToggleCommand::GetStatus => {
                let status = client.plugins.toggle.get_status().await?;
                Ok(serde_json::to_string_pretty(&status)?)
            }
            ToggleCommand::Toggle => {
                let result = client.plugins.toggle.toggle().await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum WebUiCommand {
    #[command(name = "got-deluge-web")]
    GotDelugeWeb,
    #[command(name = "get-config")]
    GetConfig,
    #[command(name = "set-config")]
    SetConfig { json: String },
}

impl WebUiCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            WebUiCommand::GotDelugeWeb => {
                let result = client.plugins.webui.got_deluge_web().await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
            WebUiCommand::GetConfig => {
                let config = client.plugins.webui.get_config().await?;
                Ok(serde_json::to_string_pretty(&config)?)
            }
            WebUiCommand::SetConfig { json } => {
                let config: WebUiConfig = serde_json::from_str(json)
                    .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;
                client.plugins.webui.set_config(&config).await?;
                Ok("Config set.".to_owned())
            }
        }
    }
}

fn parse_execute_event(event: &str) -> anyhow::Result<ExecuteEvent> {
    match event {
        "complete" => Ok(ExecuteEvent::Complete),
        "added" => Ok(ExecuteEvent::Added),
        "removed" => Ok(ExecuteEvent::Removed),
        other => anyhow::bail!("invalid event '{other}': expected complete, added, or removed"),
    }
}
