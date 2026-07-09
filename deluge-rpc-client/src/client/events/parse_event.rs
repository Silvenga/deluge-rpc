use deluge_rpc_models::DelugeEvent;
use deluge_rpc_rencode::RencodeValue;

pub fn parse_event(name: &str, args: &[RencodeValue]) -> DelugeEvent {
    match name {
        "TorrentAddedEvent" => parse_torrent_added(args),
        "PreTorrentRemovedEvent" => parse_single_str(args, |torrent_id| {
            DelugeEvent::PreTorrentRemoved { torrent_id }
        }),
        "TorrentRemovedEvent" => parse_single_str(args, |torrent_id| DelugeEvent::TorrentRemoved {
            torrent_id,
        }),
        "TorrentFinishedEvent" => parse_single_str(args, |torrent_id| {
            DelugeEvent::TorrentFinished { torrent_id }
        }),
        "TorrentResumedEvent" => parse_single_str(args, |torrent_id| DelugeEvent::TorrentResumed {
            torrent_id,
        }),
        "TorrentStateChangedEvent" => parse_state_changed(args),
        "TorrentQueueChangedEvent" => DelugeEvent::TorrentQueueChanged,

        "TorrentFileCompletedEvent" => parse_file_completed(args),
        "TorrentFileRenamedEvent" => parse_file_renamed(args),
        "TorrentFolderRenamedEvent" => parse_folder_renamed(args),
        "TorrentStorageMovedEvent" => parse_storage_moved(args),
        "TorrentTrackerStatusEvent" => parse_tracker_status(args),

        "SessionStartedEvent" => DelugeEvent::SessionStarted,
        "SessionPausedEvent" => DelugeEvent::SessionPaused,
        "SessionResumedEvent" => DelugeEvent::SessionResumed,

        "ConfigValueChangedEvent" => parse_config_changed(args),
        "PluginEnabledEvent" => parse_single_str(args, |plugin_name| DelugeEvent::PluginEnabled {
            plugin_name,
        }),
        "PluginDisabledEvent" => parse_single_str(args, |plugin_name| {
            DelugeEvent::PluginDisabled { plugin_name }
        }),

        "NewVersionAvailableEvent" => parse_single_str(args, |new_release| {
            DelugeEvent::NewVersionAvailable { new_release }
        }),
        "ClientDisconnectedEvent" => parse_single_int(args, |session_id| {
            DelugeEvent::ClientDisconnected { session_id }
        }),
        "ExternalIPEvent" => {
            parse_single_str(args, |external_ip| DelugeEvent::ExternalIP { external_ip })
        }
        "CreateTorrentProgressEvent" => parse_torrent_progress(args),

        "AutoaddOptionsChangedEvent" => DelugeEvent::AutoaddOptionsChanged,
        "ExecuteCommandAddedEvent" => parse_execute_command_added(args),
        "ExecuteCommandRemovedEvent" => parse_single_str(args, |command_id| {
            DelugeEvent::ExecuteCommandRemoved { command_id }
        }),
        "SchedulerEvent" => parse_single_str(args, |colour| DelugeEvent::SchedulerEvent { colour }),

        _ => DelugeEvent::Unknown {
            name: name.to_owned(),
            args: args.to_vec(),
        },
    }
}

fn arg_as_str(value: &RencodeValue) -> Option<String> {
    match value {
        RencodeValue::Str(s) => Some(s.clone()),
        _ => None,
    }
}

fn arg_as_i32(value: &RencodeValue) -> Option<i32> {
    match value {
        RencodeValue::Int(i) => i32::try_from(*i).ok(),
        _ => None,
    }
}

fn parse_single_str<F>(args: &[RencodeValue], f: F) -> DelugeEvent
where
    F: FnOnce(String) -> DelugeEvent,
{
    match args.first().and_then(arg_as_str) {
        Some(s) => f(s),
        None => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_single_int<F>(args: &[RencodeValue], f: F) -> DelugeEvent
where
    F: FnOnce(i32) -> DelugeEvent,
{
    match args.first().and_then(arg_as_i32) {
        Some(i) => f(i),
        None => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_torrent_added(args: &[RencodeValue]) -> DelugeEvent {
    let torrent_id = args.first().and_then(arg_as_str);
    let from_state = args.get(1).and_then(|v| match v {
        RencodeValue::Bool(b) => Some(*b),
        _ => None,
    });
    match (torrent_id, from_state) {
        (Some(torrent_id), Some(from_state)) => DelugeEvent::TorrentAdded {
            torrent_id,
            from_state,
        },
        _ => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_state_changed(args: &[RencodeValue]) -> DelugeEvent {
    let torrent_id = args.first().and_then(arg_as_str);
    let state = args.get(1).and_then(arg_as_str);
    match (torrent_id, state) {
        (Some(torrent_id), Some(state)) => DelugeEvent::TorrentStateChanged { torrent_id, state },
        _ => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_file_completed(args: &[RencodeValue]) -> DelugeEvent {
    let torrent_id = args.first().and_then(arg_as_str);
    let index = args.get(1).and_then(arg_as_i32);
    match (torrent_id, index) {
        (Some(torrent_id), Some(index)) => DelugeEvent::TorrentFileCompleted { torrent_id, index },
        _ => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_file_renamed(args: &[RencodeValue]) -> DelugeEvent {
    let torrent_id = args.first().and_then(arg_as_str);
    let index = args.get(1).and_then(arg_as_i32);
    let name = args.get(2).and_then(arg_as_str);
    match (torrent_id, index, name) {
        (Some(torrent_id), Some(index), Some(name)) => DelugeEvent::TorrentFileRenamed {
            torrent_id,
            index,
            name,
        },
        _ => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_folder_renamed(args: &[RencodeValue]) -> DelugeEvent {
    let torrent_id = args.first().and_then(arg_as_str);
    let old = args.get(1).and_then(arg_as_str);
    let new = args.get(2).and_then(arg_as_str);
    match (torrent_id, old, new) {
        (Some(torrent_id), Some(old), Some(new)) => DelugeEvent::TorrentFolderRenamed {
            torrent_id,
            old,
            new,
        },
        _ => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_storage_moved(args: &[RencodeValue]) -> DelugeEvent {
    let torrent_id = args.first().and_then(arg_as_str);
    let path = args.get(1).and_then(arg_as_str);
    match (torrent_id, path) {
        (Some(torrent_id), Some(path)) => DelugeEvent::TorrentStorageMoved { torrent_id, path },
        _ => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_tracker_status(args: &[RencodeValue]) -> DelugeEvent {
    let torrent_id = args.first().and_then(arg_as_str);
    let status = args.get(1).and_then(arg_as_str);
    match (torrent_id, status) {
        (Some(torrent_id), Some(status)) => {
            DelugeEvent::TorrentTrackerStatus { torrent_id, status }
        }
        _ => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_config_changed(args: &[RencodeValue]) -> DelugeEvent {
    let key = args.first().and_then(arg_as_str);
    let value = args.get(1).cloned();
    match (key, value) {
        (Some(key), Some(value)) => DelugeEvent::ConfigValueChanged { key, value },
        _ => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_torrent_progress(args: &[RencodeValue]) -> DelugeEvent {
    let piece_count = args.first().and_then(arg_as_i32);
    let num_pieces = args.get(1).and_then(arg_as_i32);
    match (piece_count, num_pieces) {
        (Some(piece_count), Some(num_pieces)) => DelugeEvent::CreateTorrentProgress {
            piece_count,
            num_pieces,
        },
        _ => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}

fn parse_execute_command_added(args: &[RencodeValue]) -> DelugeEvent {
    let command_id = args.first().and_then(arg_as_str);
    let event = args.get(1).and_then(arg_as_str);
    let command = args.get(2).and_then(arg_as_str);
    match (command_id, event, command) {
        (Some(command_id), Some(event), Some(command)) => DelugeEvent::ExecuteCommandAdded {
            command_id,
            event,
            command,
        },
        _ => DelugeEvent::Unknown {
            name: String::new(),
            args: args.to_vec(),
        },
    }
}
