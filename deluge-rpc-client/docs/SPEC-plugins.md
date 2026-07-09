# SPEC - Plugin Methods

> `deluge-torrent/deluge` @ commit `e58075416dedd53636e89b1cd240f86f2e7c2ee0`

> Plugin methods only exist when the plugin is enabled. Check via `core.get_enabled_plugins()` or use
`daemon.get_method_list()` to confirm registration. Calling a disabled plugin's method raises `WrappedException` (Python
`AttributeError` wrapped).

## `autoadd.` - 10 methods

Automatically adds torrents from watched directories.

| Method                     | Args                     | Returns                                           | Description                                                                                                                             |
|----------------------------|--------------------------|---------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------|
| `autoadd.set_options`      | `(watchdir_id, options)` | `None`                                            | Updates the options for an existing watch folder. `options` keys are a subset of the watchdir options (see "AutoAdd watchdir options"). |
| `autoadd.enable_watchdir`  | `(watchdir_id)`          | `None`                                            | Enables a watch folder - starts polling for new .torrent files.                                                                         |
| `autoadd.disable_watchdir` | `(watchdir_id)`          | `None`                                            | Disables a watch folder - stops polling.                                                                                                |
| `autoadd.set_config`       | `(config)`               | `None`                                            | Sets the plugin config. Keys: see "AutoAdd config".                                                                                     |
| `autoadd.get_config`       | `()`                     | `dict`                                            | Returns the plugin config. Keys: see "AutoAdd config".                                                                                  |
| `autoadd.get_watchdirs`    | `()`                     | `Dict[str, dict]` (`{watchdir_id: options_dict}`) | Returns all watch folders. Filtered by owner for non-admin users - each user sees only their own watchdirs unless they're admin.        |
| `autoadd.add`              | `(options=None)`         | `int` (new `watchdir_id`)                         | Creates a new watch folder with the given options. Returns the numeric ID of the new watchdir.                                          |
| `autoadd.remove`           | `(watchdir_id)`          | `None`                                            | Removes a watch folder.                                                                                                                 |
| `autoadd.is_admin_level`   | `()`                     | `bool`                                            | Returns `True` if the current session has admin auth level.                                                                             |
| `autoadd.get_auth_user`    | `()`                     | `str`                                             | Returns the username of the current session.                                                                                            |

### AutoAdd config

| Key         | Type              | Default | Description                          |
|-------------|-------------------|---------|--------------------------------------|
| `watchdirs` | `Dict[str, dict]` | `{}`    | All watch folders keyed by ID.       |
| `next_id`   | `int`             | `1`     | Next ID to assign to a new watchdir. |

### AutoAdd watchdir options

Each watchdir dict (in `watchdirs` or returned by `get_watchdirs`):

| Key                          | Type    | Description                                                |
|------------------------------|---------|------------------------------------------------------------|
| `enabled`                    | `bool`  | Whether the watchdir is actively polling.                  |
| `path`                       | `str`   | Filesystem path to watch for .torrent files.               |
| `append_extension`           | `str`   | Extension to append to processed .torrent files.           |
| `copy_torrent`               | `bool`  | Whether to copy .torrent files to `torrentfiles_location`. |
| `delete_copy_torrent_toggle` | `bool`  | Whether to delete copied .torrent files after processing.  |
| `abspath`                    | `bool`  | Whether to use absolute paths.                             |
| `download_location`          | `str`   | Save path for torrents added from this watchdir.           |
| `max_download_speed`         | `int`   | Per-torrent max download speed.                            |
| `max_upload_speed`           | `int`   | Per-torrent max upload speed.                              |
| `max_connections`            | `int`   | Per-torrent max connections.                               |
| `max_upload_slots`           | `int`   | Per-torrent max upload slots.                              |
| `prioritize_first_last`      | `bool`  | Whether to prioritize first/last pieces.                   |
| `auto_managed`               | `bool`  | Whether torrents are auto-managed.                         |
| `stop_at_ratio`              | `bool`  | Whether to stop at stop ratio.                             |
| `stop_ratio`                 | `float` | The stop ratio.                                            |
| `remove_at_ratio`            | `bool`  | Whether to remove at stop ratio.                           |
| `move_completed`             | `bool`  | Whether to move on completion.                             |
| `move_completed_path`        | `str`   | Move-on-completion destination.                            |
| `label`                      | `str`   | Label to apply (requires Label plugin).                    |
| `add_paused`                 | `bool`  | Whether to add torrents paused.                            |
| `queue_to_top`               | `bool`  | Whether to place new torrents at queue top.                |
| `owner`                      | `str`   | Username of the watchdir owner.                            |
| `seed_mode`                  | `bool`  | Whether to add torrents in seed mode.                      |

Plus `_toggle` variants for most options (e.g. `max_download_speed_toggle`, `download_location_toggle`) - boolean flags
controlling whether the option is applied when adding.

---

## `blocklist.` - 4 methods

Downloads and imports IP blocklists into libtorrent.

| Method                   | Args            | Returns                                         | Description                                                                                                                                                                                                            |
|--------------------------|-----------------|-------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `blocklist.check_import` | `(force=False)` | `Deferred[str]` (blocklist file path) or `None` | Downloads and imports the blocklist from the configured URL. `force=True` re-imports even if already up-to-date. Returns `None` if no URL is configured. The Deferred resolves to the downloaded file path on success. |
| `blocklist.get_config`   | `()`            | `dict`                                          | Returns the plugin config. Keys: see "Blocklist config".                                                                                                                                                               |
| `blocklist.set_config`   | `(config)`      | `None`                                          | Sets the plugin config. May trigger a re-import if the URL changed.                                                                                                                                                    |
| `blocklist.get_status`   | `()`            | `dict`                                          | Returns the current import status. Keys: see "Blocklist status".                                                                                                                                                       |

### Blocklist config

| Key                | Type        | Default  | Description                                         |
|--------------------|-------------|----------|-----------------------------------------------------|
| `url`              | `str`       | `''`     | Blocklist download URL.                             |
| `load_on_start`    | `bool`      | `True`   | Whether to load the blocklist on daemon startup.    |
| `check_after_days` | `int`       | `4`      | Days between automatic re-import checks.            |
| `list_compression` | `str`       | `''`     | Compression type (e.g. `'gzip'`).                   |
| `list_type`        | `str`       | `'text'` | Blocklist format (e.g. `'text'`, `'p2p'`, `'dat'`). |
| `last_update`      | `float`     | `0.0`    | Unix timestamp of last successful import.           |
| `list_size`        | `int`       | `0`      | Number of entries in the current blocklist.         |
| `timeout`          | `int`       | `30`     | Download timeout in seconds.                        |
| `try_times`        | `int`       | `3`      | Number of retry attempts on download failure.       |
| `whitelisted`      | `List[str]` | `[]`     | Whitelisted IPs that bypass the blocklist.          |

### Blocklist status

| Key             | Type        | Description                                                 |
|-----------------|-------------|-------------------------------------------------------------|
| `state`         | `str`       | Current state: `"Downloading"`, `"Importing"`, or `"Idle"`. |
| `up_to_date`    | `bool`      | Whether the blocklist is current.                           |
| `num_whited`    | `int`       | Number of whitelisted IPs.                                  |
| `num_blocked`   | `int`       | Number of blocked IPs/ranges.                               |
| `file_progress` | `float`     | Download progress `0.0`–`1.0`.                              |
| `file_url`      | `str`       | URL of the current/last download.                           |
| `file_size`     | `int`       | Size of the downloaded file in bytes.                       |
| `file_date`     | `float`     | Unix timestamp of the downloaded file.                      |
| `file_type`     | `str`       | Blocklist type (e.g. `"p2p"`, `"p2p (gz)"`).                |
| `whitelisted`   | `List[str]` | Whitelisted IPs.                                            |

---

## `execute.` - 4 methods

Runs shell commands when torrent events occur.

| Method                   | Args                       | Returns                      | Description                                                                                                                                                                                                                      |
|--------------------------|----------------------------|------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `execute.add_command`    | `(event, command)`         | `None`                       | Adds a command to run on a torrent event. `event` is `"complete"`, `"added"`, or `"removed"`. `command` is a shell command string. The command_id is generated server-side and not returned - use `get_commands` to discover it. |
| `execute.get_commands`   | `()`                       | `List[Tuple[str, str, str]]` | Returns all commands. Each tuple: `(command_id, event, command)`. `command_id` is a SHA1 hex string. `event` is `"complete"`, `"added"`, or `"removed"`.                                                                         |
| `execute.remove_command` | `(command_id)`             | `None`                       | Removes a command by its ID.                                                                                                                                                                                                     |
| `execute.save_command`   | `(command_id, event, cmd)` | `None`                       | Updates an existing command in-place by ID.                                                                                                                                                                                      |

---

## `extractor.` - 2 methods

Automatically extracts archived files from completed torrents.

| Method                 | Args       | Returns | Description                                              |
|------------------------|------------|---------|----------------------------------------------------------|
| `extractor.set_config` | `(config)` | `None`  | Sets the plugin config. Keys: see "Extractor config".    |
| `extractor.get_config` | `()`       | `dict`  | Returns the plugin config. Keys: see "Extractor config". |

### Extractor config

| Key               | Type   | Default | Description                                                         |
|-------------------|--------|---------|---------------------------------------------------------------------|
| `extract_path`    | `str`  | `''`    | Destination path for extracted files. Empty means extract in-place. |
| `use_name_folder` | `bool` | `True`  | Whether to extract into a folder named after the torrent.           |

---

## `label.` - 8 methods

Assigns labels to torrents and applies per-label options.

| Method              | Args                       | Returns                        | Description                                                                                               |
|---------------------|----------------------------|--------------------------------|-----------------------------------------------------------------------------------------------------------|
| `label.get_labels`  | `()`                       | `List[str]` (sorted label IDs) | Returns all label IDs.                                                                                    |
| `label.add`         | `(label_id)`               | `None`                         | Creates a new label with default options.                                                                 |
| `label.remove`      | `(label_id)`               | `None`                         | Removes a label. Torrents with that label lose it.                                                        |
| `label.set_options` | `(label_id, options_dict)` | `None`                         | Updates a label's options and re-applies them to all torrents with that label. Keys: see "Label options". |
| `label.get_options` | `(label_id)`               | `dict`                         | Returns a label's options. Keys: see "Label options".                                                     |
| `label.set_torrent` | `(torrent_id, label_id)`   | `None`                         | Assigns a label to a torrent. Passing `"No Label"` as `label_id` removes the label.                       |
| `label.get_config`  | `()`                       | `dict`                         | Returns the plugin's global config. Contains `auto_add_trackers` when configured; the key may be absent if no trackers are set. |
| `label.set_config`  | `(options)`                | `None`                         | Sets the plugin's global config. Only `auto_add_trackers` is settable.                                    |

### Label options

| Key                     | Type        | Default | Description                                        |
|-------------------------|-------------|---------|----------------------------------------------------|
| `apply_max`             | `bool`      | `False` | Whether to apply bandwidth limits.                 |
| `max_download_speed`    | `int`       | `-1`    | Max download speed; `-1` = unlimited.              |
| `max_upload_speed`      | `int`       | `-1`    | Max upload speed; `-1` = unlimited.                |
| `max_connections`       | `int`       | `-1`    | Max connections; `-1` = unlimited.                 |
| `max_upload_slots`      | `int`       | `-1`    | Max upload slots; `-1` = unlimited.                |
| `prioritize_first_last` | `bool`      | `False` | Whether to prioritize first/last pieces.           |
| `apply_queue`           | `bool`      | `False` | Whether to apply queue settings.                   |
| `is_auto_managed`       | `bool`      | `False` | Whether torrents with this label are auto-managed. |
| `stop_at_ratio`         | `bool`      | `False` | Whether to stop at stop ratio.                     |
| `stop_ratio`            | `float`     | `2.0`   | The stop ratio.                                    |
| `remove_at_ratio`       | `bool`      | `False` | Whether to remove at stop ratio.                   |
| `apply_move_completed`  | `bool`      | `False` | Whether to apply move-on-completion settings.      |
| `move_completed`        | `bool`      | `False` | Whether to move on completion.                     |
| `move_completed_path`   | `str`       | `''`    | Move-on-completion destination.                    |
| `auto_add`              | `bool`      | `False` | Whether to auto-add torrents matching trackers.    |
| `auto_add_trackers`     | `List[str]` | `[]`    | Tracker URLs that trigger auto-adding.             |

---

## `notifications.` - 3 methods

Sends email notifications for torrent events.

| Method                             | Args       | Returns                 | Description                                                                                                                                            |
|------------------------------------|------------|-------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| `notifications.set_config`         | `(config)` | `None`                  | Sets the plugin config. Keys: see "Notifications config".                                                                                              |
| `notifications.get_config`         | `()`       | `dict`                  | Returns the plugin config. Keys: see "Notifications config".                                                                                           |
| `notifications.get_handled_events` | `()`       | `List[Tuple[str, str]]` | Returns events that the plugin can handle. Each tuple: `(event_name, class_docstring)`. Excludes built-in Deluge events except `TorrentFinishedEvent`. |

### Notifications config

| Key               | Type        | Default         | Description                                |
|-------------------|-------------|-----------------|--------------------------------------------|
| `smtp_enabled`    | `bool`      | `False`         | Whether email notifications are enabled.   |
| `smtp_host`       | `str`       | `''`            | SMTP server hostname.                      |
| `smtp_port`       | `int`       | `25`            | SMTP server port.                          |
| `smtp_user`       | `str`       | `''`            | SMTP username.                             |
| `smtp_pass`       | `str`       | `''`            | SMTP password.                             |
| `smtp_from`       | `str`       | `''`            | From address for notification emails.      |
| `smtp_tls`        | `bool`      | `False`         | Whether to use TLS for SMTP.               |
| `smtp_recipients` | `List[str]` | `[]`            | Email addresses to notify.                 |
| `subscriptions`   | `dict`      | `{'email': []}` | Event subscriptions per notification type. |

---

## `scheduler.` - 3 methods

Schedules bandwidth limits by time of day / day of week.

| Method                 | Args       | Returns | Description                                                                                                                                                          |
|------------------------|------------|---------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `scheduler.set_config` | `(config)` | `None`  | Sets the plugin config and re-runs the scheduler. Keys: see "Scheduler config".                                                                                      |
| `scheduler.get_config` | `()`       | `dict`  | Returns the plugin config. Keys: see "Scheduler config".                                                                                                             |
| `scheduler.get_state`  | `()`       | `str`   | Returns the current schedule state: `"Green"` (unlimited), `"Yellow"` (limited), or `"Red"` (stopped). Determined by the current hour and weekday in `button_state`. |

### Scheduler config

| Key               | Type              | Default   | Description                                                                         |
|-------------------|-------------------|-----------|-------------------------------------------------------------------------------------|
| `low_down`        | `float`           | `-1.0`    | Download speed limit in "Yellow" state (KiB/s); `-1.0` = unlimited.                 |
| `low_up`          | `float`           | `-1.0`    | Upload speed limit in "Yellow" state (KiB/s); `-1.0` = unlimited.                   |
| `low_active`      | `int`             | `-1`      | Max active torrents in "Yellow" state; `-1` = unlimited.                            |
| `low_active_down` | `int`             | `-1`      | Max active downloads in "Yellow" state; `-1` = unlimited.                           |
| `low_active_up`   | `int`             | `-1`      | Max active seeds in "Yellow" state; `-1` = unlimited.                               |
| `button_state`    | `List[List[int]]` | 24×7 grid | Schedule grid indexed as `[hour][weekday]`. Values: `0`=Green, `1`=Yellow, `2`=Red. |

---

## `stats.` - 6 methods

Collects and returns historical session statistics.

| Method                     | Args               | Returns          | Description                                                                                                                           |
|----------------------------|--------------------|------------------|---------------------------------------------------------------------------------------------------------------------------------------|
| `stats.get_stats`          | `(keys, interval)` | `dict` or `None` | Returns historical stats for the requested keys at the given interval. `None` if `interval` is invalid. See "Stats get_stats return". |
| `stats.get_totals`         | `()`               | `dict`           | Returns cumulative totals (persisted + current session). Keys: see "Stats totals".                                                    |
| `stats.get_session_totals` | `()`               | `dict`           | Returns current session totals. Keys: see "Stats totals".                                                                             |
| `stats.set_config`         | `(config)`         | `None`           | Sets the plugin config. Keys: see "Stats config".                                                                                     |
| `stats.get_config`         | `()`               | `dict`           | Returns the plugin config. Keys: see "Stats config".                                                                                  |
| `stats.get_intervals`      | `()`               | `List[int]`      | Returns valid sampling intervals: `[1, 5, 30, 300]` (seconds).                                                                        |

### Stats get_stats return

Shape:

```python
{key_name: List[int], '_last_update': float, '_length': int, '_update_interval': int}
```

Available `keys`:
`upload_rate`, `download_rate`, `dht_nodes`, `dht_cache_nodes`, `dht_torrents`, `num_peers`, `num_connections`

Each `key_name` maps to a list of historical samples (ints). Valid `interval` values: `1`, `5`, `30`, `300` (seconds).

### Stats totals

Returned by `get_totals` and `get_session_totals` (`get_totals` adds persisted totals to session totals):

| Key                      | Type  | Description                     |
|--------------------------|-------|---------------------------------|
| `total_upload`           | `int` | Total bytes uploaded.           |
| `total_download`         | `int` | Total bytes downloaded.         |
| `total_payload_upload`   | `int` | Total payload bytes uploaded.   |
| `total_payload_download` | `int` | Total payload bytes downloaded. |

### Stats config

| Key               | Type  | Default    | Description                   |
|-------------------|-------|------------|-------------------------------|
| `test`            | `str` | `"NiNiNi"` | Test key (unused).            |
| `update_interval` | `int` | `1`        | Sampling interval in seconds. |
| `length`          | `int` | `150`      | Number of samples to keep.    |

---

## `toggle.` - 2 methods

Toggles the session pause state.

| Method              | Args | Returns | Description                                                                   |
|---------------------|------|---------|-------------------------------------------------------------------------------|
| `toggle.get_status` | `()` | `bool`  | Returns `True` if the session is paused.                                      |
| `toggle.toggle`     | `()` | `bool`  | Toggles the session between paused and running. Returns the new paused state. |

---

## `webui.` - 3 methods

Manages the embedded web UI server.

| Method                 | Args       | Returns              | Description                                                                                                                                                                                                                                          |
|------------------------|------------|----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `webui.got_deluge_web` | `()`       | `bool`               | Returns `True` if the `deluge-web` module is installed and importable.                                                                                                                                                                               |
| `webui.set_config`     | `(config)` | `None` or `Deferred` | Sets the plugin config. If `enabled` or `ssl` changed, starts/stops/restarts the web server. Return type depends on the action: start -> `bool` or raises `CannotListenError`; stop -> `Deferred[True]`; restart -> `Deferred`; no action -> `None`. |
| `webui.get_config`     | `()`       | `dict`               | Returns the plugin config. Keys: see "WebUi config".                                                                                                                                                                                                 |

### WebUi config

| Key       | Type   | Default | Description                           |
|-----------|--------|---------|---------------------------------------|
| `enabled` | `bool` | `False` | Whether the web UI server is enabled. |
| `ssl`     | `bool` | `False` | Whether the web UI uses SSL.          |
| `port`    | `int`  | `8112`  | Port for the web UI server.           |
