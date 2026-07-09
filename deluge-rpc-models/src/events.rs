use deluge_rpc_rencode::RencodeValue;

/// A typed Deluge daemon event.
#[derive(Debug, Clone)]
pub enum DelugeEvent {
    /// A new torrent was added to the session.
    TorrentAdded {
        /// The ID of the added torrent.
        torrent_id: String,
        /// Whether the torrent was loaded from saved state.
        from_state: bool,
    },
    /// A torrent is about to be removed.
    PreTorrentRemoved {
        /// The ID of the torrent being removed.
        torrent_id: String,
    },
    /// A torrent has been removed.
    TorrentRemoved {
        /// The ID of the removed torrent.
        torrent_id: String,
    },
    /// A torrent finished downloading.
    TorrentFinished {
        /// The ID of the finished torrent.
        torrent_id: String,
    },
    /// A torrent resumed from a paused state.
    TorrentResumed {
        /// The ID of the resumed torrent.
        torrent_id: String,
    },
    /// A torrent's state changed (e.g. Downloading to Seeding).
    TorrentStateChanged {
        /// The ID of the torrent.
        torrent_id: String,
        /// The new state.
        state: String,
    },
    /// The queue order changed.
    TorrentQueueChanged,
    /// An individual file within a torrent completed.
    TorrentFileCompleted {
        /// The ID of the torrent.
        torrent_id: String,
        /// The index of the completed file.
        index: i32,
    },
    /// A file within a torrent was renamed.
    TorrentFileRenamed {
        /// The ID of the torrent.
        torrent_id: String,
        /// The index of the renamed file.
        index: i32,
        /// The new name.
        name: String,
    },
    /// A folder within a torrent was renamed.
    TorrentFolderRenamed {
        /// The ID of the torrent.
        torrent_id: String,
        /// The old folder name.
        old: String,
        /// The new folder name.
        new: String,
    },
    /// A torrent's storage was moved. Defined in Deluge but not currently emitted.
    TorrentStorageMoved {
        /// The ID of the torrent.
        torrent_id: String,
        /// The new storage path.
        path: String,
    },
    /// A torrent's tracker status changed.
    TorrentTrackerStatus {
        /// The ID of the torrent.
        torrent_id: String,
        /// The new tracker status.
        status: String,
    },
    /// The session started.
    SessionStarted,
    /// The session was paused.
    SessionPaused,
    /// The session was resumed.
    SessionResumed,
    /// A config value changed.
    ConfigValueChanged {
        /// The config key that changed.
        key: String,
        /// The new value.
        value: RencodeValue,
    },
    /// A plugin was enabled.
    PluginEnabled {
        /// The name of the enabled plugin.
        plugin_name: String,
    },
    /// A plugin was disabled.
    PluginDisabled {
        /// The name of the disabled plugin.
        plugin_name: String,
    },
    /// A new version of Deluge is available.
    NewVersionAvailable {
        /// The new release version string.
        new_release: String,
    },
    /// A client disconnected from the daemon.
    ClientDisconnected {
        /// The session ID of the disconnected client.
        session_id: i32,
    },
    /// The external IP address was received from libtorrent.
    ExternalIP {
        /// The external IP address.
        external_ip: String,
    },
    /// Progress update while creating a torrent file.
    CreateTorrentProgress {
        /// The number of pieces processed so far.
        piece_count: i32,
        /// The total number of pieces.
        num_pieces: i32,
    },
    /// AutoAdd plugin options changed.
    AutoaddOptionsChanged,
    /// An Execute command was added.
    ExecuteCommandAdded {
        /// The command ID.
        command_id: String,
        /// The event that triggers the command.
        event: String,
        /// The command string to execute.
        command: String,
    },
    /// An Execute command was removed.
    ExecuteCommandRemoved {
        /// The command ID.
        command_id: String,
    },
    /// The Scheduler plugin state changed.
    SchedulerEvent {
        /// The scheduler colour: `"Green"`, `"Yellow"`, or `"Red"`.
        colour: String,
    },
    /// An event not yet modeled in typed form.
    Unknown {
        /// The raw event name from the wire.
        name: String,
        /// The raw event arguments.
        args: Vec<RencodeValue>,
    },
}
