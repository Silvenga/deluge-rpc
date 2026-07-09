use clap::Subcommand;
use deluge_rpc_client::DelugeClient;
use deluge_rpc_client::models::DelugeEvent;
use std::time::Duration;
use tokio::time::timeout as tokio_timeout;
use tokio_stream::StreamExt;
use tracing::info;

/// `daemon.*` methods (e.g., daemon info, version, and shutdown).
#[derive(Subcommand, Debug, Clone)]
pub enum DaemonCommand {
    /// Get the daemon version string (pre-auth handshake value, same as `daemon.get_version`).
    Info,
    /// Get the daemon version string (e.g. `"2.1.1"`).
    Version,
    /// List all registered RPC method names (`"<object>.<method>"` format).
    /// Only includes `@export`-registered methods - excludes the hardcoded
    /// `daemon.info`, `daemon.login`, and `daemon.set_event_interest`.
    Methods,
    /// Shut down the daemon. The response is unreliable.
    Shutdown,
    /// Subscribe the session to event names (full-replace operation).
    /// Stream events from a dedicated connection until timeout or interrupted.
    /// Subscribes to the given event names and prints each event as JSON.
    Events {
        /// JSON array of event names to subscribe to (e.g. `["TorrentAddedEvent"]`).
        names: String,
        /// Timeout in seconds. `None` (default) subscribes forever until interrupted.
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Check whether the current session can call a named RPC method.
    Authorized {
        /// The RPC method name to check (e.g. `core.get_config`).
        rpc: String,
    },
}

impl DaemonCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        match self {
            DaemonCommand::Info => {
                let info = client.daemon.info().await?;
                Ok(serde_json::to_string_pretty(&info)?)
            }
            DaemonCommand::Version => {
                let version = client.daemon.get_version().await?;
                Ok(serde_json::to_string_pretty(&version)?)
            }
            DaemonCommand::Methods => {
                let methods = client.daemon.get_method_list().await?;
                Ok(serde_json::to_string_pretty(&methods)?)
            }
            DaemonCommand::Shutdown => {
                client.daemon.shutdown().await?;
                Ok("Shutdown requested.".to_owned())
            }
            DaemonCommand::Events { names, timeout } => {
                let event_names: Vec<String> = serde_json::from_str(names)
                    .map_err(|e| anyhow::anyhow!("failed to parse event names JSON: {e}"))?;
                let mut stream = client.subscribe_events(&event_names).await?;

                let event_loop = async {
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(event) => {
                                let json = event_to_json(&event);
                                info!("{json}");
                            }
                            Err(e) => {
                                anyhow::bail!("event stream error: {e}");
                            }
                        }
                    }
                    Ok::<(), anyhow::Error>(())
                };

                match timeout {
                    Some(secs) => {
                        tokio_timeout(Duration::from_secs(*secs), event_loop).await??;
                    }
                    None => {
                        event_loop.await?;
                    }
                }

                Ok(String::new())
            }
            DaemonCommand::Authorized { rpc } => {
                let result = client.daemon.authorized_call(rpc).await?;
                Ok(serde_json::to_string_pretty(&result)?)
            }
        }
    }
}

fn event_to_json(event: &DelugeEvent) -> serde_json::Value {
    use serde_json::json;
    match event {
        DelugeEvent::TorrentAdded {
            torrent_id,
            from_state,
        } => {
            json!({"type": "TorrentAddedEvent", "torrent_id": torrent_id, "from_state": from_state})
        }
        DelugeEvent::PreTorrentRemoved { torrent_id } => {
            json!({"type": "PreTorrentRemovedEvent", "torrent_id": torrent_id})
        }
        DelugeEvent::TorrentRemoved { torrent_id } => {
            json!({"type": "TorrentRemovedEvent", "torrent_id": torrent_id})
        }
        DelugeEvent::TorrentFinished { torrent_id } => {
            json!({"type": "TorrentFinishedEvent", "torrent_id": torrent_id})
        }
        DelugeEvent::TorrentResumed { torrent_id } => {
            json!({"type": "TorrentResumedEvent", "torrent_id": torrent_id})
        }
        DelugeEvent::TorrentStateChanged { torrent_id, state } => {
            json!({"type": "TorrentStateChangedEvent", "torrent_id": torrent_id, "state": state})
        }
        DelugeEvent::TorrentQueueChanged => {
            json!({"type": "TorrentQueueChangedEvent"})
        }
        DelugeEvent::TorrentFileCompleted { torrent_id, index } => {
            json!({"type": "TorrentFileCompletedEvent", "torrent_id": torrent_id, "index": index})
        }
        DelugeEvent::TorrentFileRenamed {
            torrent_id,
            index,
            name,
        } => {
            json!({"type": "TorrentFileRenamedEvent", "torrent_id": torrent_id, "index": index, "name": name})
        }
        DelugeEvent::TorrentFolderRenamed {
            torrent_id,
            old,
            new,
        } => {
            json!({"type": "TorrentFolderRenamedEvent", "torrent_id": torrent_id, "old": old, "new": new})
        }
        DelugeEvent::TorrentStorageMoved { torrent_id, path } => {
            json!({"type": "TorrentStorageMovedEvent", "torrent_id": torrent_id, "path": path})
        }
        DelugeEvent::TorrentTrackerStatus { torrent_id, status } => {
            json!({"type": "TorrentTrackerStatusEvent", "torrent_id": torrent_id, "status": status})
        }
        DelugeEvent::SessionStarted => {
            json!({"type": "SessionStartedEvent"})
        }
        DelugeEvent::SessionPaused => {
            json!({"type": "SessionPausedEvent"})
        }
        DelugeEvent::SessionResumed => {
            json!({"type": "SessionResumedEvent"})
        }
        DelugeEvent::ConfigValueChanged { key, value } => {
            json!({"type": "ConfigValueChangedEvent", "key": key, "value": format!("{value:?}")})
        }
        DelugeEvent::PluginEnabled { plugin_name } => {
            json!({"type": "PluginEnabledEvent", "plugin_name": plugin_name})
        }
        DelugeEvent::PluginDisabled { plugin_name } => {
            json!({"type": "PluginDisabledEvent", "plugin_name": plugin_name})
        }
        DelugeEvent::NewVersionAvailable { new_release } => {
            json!({"type": "NewVersionAvailableEvent", "new_release": new_release})
        }
        DelugeEvent::ClientDisconnected { session_id } => {
            json!({"type": "ClientDisconnectedEvent", "session_id": session_id})
        }
        DelugeEvent::ExternalIP { external_ip } => {
            json!({"type": "ExternalIPEvent", "external_ip": external_ip})
        }
        DelugeEvent::CreateTorrentProgress {
            piece_count,
            num_pieces,
        } => {
            json!({"type": "CreateTorrentProgressEvent", "piece_count": piece_count, "num_pieces": num_pieces})
        }
        DelugeEvent::AutoaddOptionsChanged => {
            json!({"type": "AutoaddOptionsChangedEvent"})
        }
        DelugeEvent::ExecuteCommandAdded {
            command_id,
            event,
            command,
        } => {
            json!({"type": "ExecuteCommandAddedEvent", "command_id": command_id, "event": event, "command": command})
        }
        DelugeEvent::ExecuteCommandRemoved { command_id } => {
            json!({"type": "ExecuteCommandRemovedEvent", "command_id": command_id})
        }
        DelugeEvent::SchedulerEvent { colour } => {
            json!({"type": "SchedulerEvent", "colour": colour})
        }
        DelugeEvent::Unknown { name, args } => {
            json!({"type": name, "args": format!("{args:?}")})
        }
    }
}
