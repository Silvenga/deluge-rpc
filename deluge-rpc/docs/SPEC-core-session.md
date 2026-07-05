# SPEC - `core.*` Session/Config/Network Methods

> `deluge-torrent/deluge` @ commit `e58075416dedd53636e89b1cd240f86f2e7c2ee0`

## Methods

### Session control

| Method                        | Args                 | Returns                        | Auth   | Description                                                                                                                                             |
|-------------------------------|----------------------|--------------------------------|--------|---------------------------------------------------------------------------------------------------------------------------------------------------------|
| `core.pause_session`          | `()`                 | `None`                         | NORMAL | Pauses the entire session (all torrents). Emits `SessionPausedEvent`.                                                                                   |
| `core.resume_session`         | `()`                 | `None`                         | NORMAL | Resumes the entire session. Emits `SessionResumedEvent`.                                                                                                |
| `core.is_session_paused`      | `()`                 | `bool`                         | NORMAL | Returns whether the session is paused.                                                                                                                  |
| `core.get_listen_port`        | `()`                 | `int`                          | NORMAL | Returns the active listen port for incoming connections.                                                                                                |
| `core.get_ssl_listen_port`    | `()`                 | `int`                          | NORMAL | Returns the active SSL listen port. `-1` if libtorrent < 2.0.10.                                                                                        |
| `core.get_external_ip`        | `()`                 | `str`                          | NORMAL | Returns the external IP address as determined by libtorrent.                                                                                            |
| `core.get_libtorrent_version` | `()`                 | `str`                          | NORMAL | Returns the libtorrent version string.                                                                                                                  |
| `core.test_listen_port`       | `()`                 | `Deferred[Optional[bool]]`     | NORMAL | Tests whether the active listen port is open by making an HTTP request to a Deluge test service. Returns `True`/`False`/`None` (on error). May be slow. |
| `core.get_session_status`     | `(keys: List[str])`  | `Dict[str, Union[int, float]]` | NORMAL | Returns libtorrent session statistics for the requested keys. `keys` is a required positional argument — pass an empty list `[]` to return all keys. See "Session status keys".                                      |
| `core.get_free_space`         | `(path: str = None)` | `int` (bytes)                  | NORMAL | Returns free space in bytes at `path`. Negative on error. `None` uses the default download location.                                                    |

### Config

| Method                   | Args                       | Returns                                   | Auth   | Description                                                                                                                                                                                           |
|--------------------------|----------------------------|-------------------------------------------|--------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `core.get_config`        | `()`                       | `dict`                                    | NORMAL | Returns all config preferences. See "Config keys".                                                                                                                                                    |
| `core.get_config_value`  | `(key: str)`               | `Any` (`None` for unknown key)            | NORMAL | Returns a single config value.                                                                                                                                                                        |
| `core.get_config_values` | `(keys: List[str])`        | `Dict[str, Any]` (missing keys -> `None`) | NORMAL | Returns a subset of config values.                                                                                                                                                                    |
| `core.set_config`        | `(config: Dict[str, Any])` | `None`                                    | NORMAL | Sets config values from a dictionary. Keys in `read_only_config_keys` (daemon CLI flag, default empty) are skipped. Type coercion is enforced - mismatched types raise `WrappedException`.            |
| `core.get_proxy`         | `()`                       | `Dict[str, Any]`                          | NORMAL | Returns live proxy settings from the libtorrent session. See "Proxy return dict". Note: this reads live session settings, NOT the config - use `get_config_value('proxy')` for the configured values. |

### Plugins

| Method                       | Args                                           | Returns          | Auth   | Description                                                                                                          |
|------------------------------|------------------------------------------------|------------------|--------|----------------------------------------------------------------------------------------------------------------------|
| `core.get_available_plugins` | `()`                                           | `List[str]`      | NORMAL | Returns names of all plugins available on the daemon (installed but not necessarily enabled).                        |
| `core.get_enabled_plugins`   | `()`                                           | `List[str]`      | NORMAL | Returns names of currently enabled plugins.                                                                          |
| `core.enable_plugin`         | `(plugin: str)`                                | `Deferred[bool]` | NORMAL | Enables a plugin. Returns `True` on success or if already enabled.                                                   |
| `core.disable_plugin`        | `(plugin: str)`                                | `Deferred[bool]` | NORMAL | Disables a plugin. Returns `True` on success or if already disabled.                                                 |
| `core.upload_plugin`         | `(filename: str, filedump: Union[str, bytes])` | `None`           | NORMAL | Uploads and installs a new plugin. `filedump` is base64-encoded plugin data. Triggers a plugin rescan after install. |
| `core.rescan_plugins`        | `()`                                           | `None`           | NORMAL | Rescans the plugin folders for newly installed plugins.                                                              |

### Accounts / auth

| Method                          | Args                                             | Returns                                 | Auth      | Description                                                                                                  |
|---------------------------------|--------------------------------------------------|-----------------------------------------|-----------|--------------------------------------------------------------------------------------------------------------|
| `core.get_known_accounts`       | `()`                                             | `List[Dict[str, Any]]`                  | **ADMIN** | Returns all known user accounts. See "Account dict".                                                         |
| `core.create_account`           | `(username: str, password: str, authlevel: str)` | `bool`                                  | **ADMIN** | Creates a new user account. `authlevel` is a string name (`NONE`, `READONLY`, `DEFAULT`, `NORMAL`, `ADMIN`). |
| `core.update_account`           | `(username: str, password: str, authlevel: str)` | `bool`                                  | **ADMIN** | Updates an existing account's password and/or auth level.                                                    |
| `core.remove_account`           | `(username: str)`                                | `bool`                                  | **ADMIN** | Removes a user account.                                                                                      |
| `core.get_auth_levels_mappings` | `()`                                             | `Tuple[Dict[str, int], Dict[int, str]]` | **NONE**  | Returns auth level name->int and int->name mappings. See SPEC.md 3.                                          |

### Torrent creation / misc

| Method                      | Args                                                                                                                                                                                                   | Returns                     | Auth   | Description                                                                                                                                                                                      |
|-----------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------|--------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `core.create_torrent`       | `(path, tracker, piece_length, comment=None, target=None, webseeds=None, private=False, created_by=None, trackers=None, add_to_session=False, torrent_format=metafile.TorrentFormat.V1, ca_cert=None)` | `Deferred[Tuple[str, str]]` | NORMAL | Creates a torrent file from `path`. Returns `(filename, filedump)` where `filedump` is base64-encoded bencoded torrent data. If `add_to_session=True`, the torrent is also added to the session. |
| `core.glob`                 | `(path: str)`                                                                                                                                                                                          | `List[str]`                 | NORMAL | Returns filesystem paths matching the glob pattern.                                                                                                                                              |
| `core.get_completion_paths` | `(args: Dict[str, Any])`                                                                                                                                                                               | `Dict[str, Any]`            | NORMAL | Returns path completions for a partial path input. See "Completion paths".                                                                                                                       |

---

## Config keys

Returned by `core.get_config()` and `core.get_config_value(key)`.

### Info / telemetry

| Key         | Type    | Default | Description                                             |
|-------------|---------|---------|---------------------------------------------------------|
| `send_info` | `bool`  | `False` | Whether to send anonymous usage info to Deluge project. |
| `info_sent` | `float` | `0.0`   | Timestamp of last info send.                            |

### Daemon / remote

| Key            | Type   | Default | Description                             |
|----------------|--------|---------|-----------------------------------------|
| `daemon_port`  | `int`  | `58846` | Port the daemon listens on.             |
| `allow_remote` | `bool` | `False` | Whether remote connections are allowed. |

### Storage / file management

| Key                            | Type        | Default              | Description                                                      |
|--------------------------------|-------------|----------------------|------------------------------------------------------------------|
| `pre_allocate_storage`         | `bool`      | `False`              | Whether to pre-allocate disk space for torrents.                 |
| `download_location`            | `str`       | default download dir | Default save path for torrents.                                  |
| `copy_torrent_file`            | `bool`      | `False`              | Whether to copy .torrent files to `torrentfiles_location`.       |
| `del_copy_torrent_file`        | `bool`      | `False`              | Whether to delete copied .torrent files when torrent is removed. |
| `torrentfiles_location`        | `str`       | default download dir | Directory for copied .torrent files.                             |
| `plugins_location`             | `str`       | config_dir/plugins   | Directory containing plugins.                                    |
| `move_completed`               | `bool`      | `False`              | Whether to move torrents when completed.                         |
| `move_completed_path`          | `str`       | default download dir | Default move-on-completion destination.                          |
| `move_completed_paths_list`    | `List[str]` | `[]`                 | History of move-completed paths.                                 |
| `download_location_paths_list` | `List[str]` | `[]`                 | History of download locations.                                   |

### Network / listening

| Key                     | Type                | Default                       | Description                                     |
|-------------------------|---------------------|-------------------------------|-------------------------------------------------|
| `listen_ports`          | `List[int]` (len 2) | `[6881, 6891]`                | Inclusive port range for incoming connections.  |
| `listen_interface`      | `str`               | `''`                          | Network interface to bind to.                   |
| `outgoing_interface`    | `str`               | `''`                          | Network interface for outgoing connections.     |
| `random_port`           | `bool`              | `True`                        | Whether to pick a random port on startup.       |
| `listen_random_port`    | `int\|None`         | `None`                        | The random port chosen (if `random_port=True`). |
| `listen_use_sys_port`   | `bool`              | `False`                       | Whether to allow system ports (<1024).          |
| `listen_reuse_port`     | `bool`              | `True`                        | Whether to set SO_REUSEPORT.                    |
| `ssl_torrents`          | `bool`              | `False`                       | Whether SSL torrents are enabled.               |
| `ssl_listen_ports`      | `List[int]` (len 2) | `[6892, 6896]`                | Inclusive port range for SSL torrents.          |
| `ssl_torrents_certs`    | `str`               | config_dir/ssl_torrents_certs | Path to SSL certs for SSL torrents.             |
| `outgoing_ports`        | `List[int]` (len 2) | `[0, 0]`                      | Inclusive port range for outgoing connections.  |
| `random_outgoing_ports` | `bool`              | `True`                        | Whether to pick random outgoing ports.          |

### DHT / PEX / LSD / trackers

| Key                     | Type   | Default | Description                                 |
|-------------------------|--------|---------|---------------------------------------------|
| `dht`                   | `bool` | `True`  | Whether DHT is enabled.                     |
| `upnp`                  | `bool` | `True`  | Whether UPnP NAT traversal is enabled.      |
| `natpmp`                | `bool` | `True`  | Whether NAT-PMP is enabled.                 |
| `utpex`                 | `bool` | `True`  | Whether uTP peer exchange is enabled.       |
| `lsd`                   | `bool` | `True`  | Whether Local Service Discovery is enabled. |
| `announce_to_all_tiers` | `bool` | `False` | Whether to announce to all tracker tiers.   |

### Encryption

| Key              | Type        | Default | Description                                                        |
|------------------|-------------|---------|--------------------------------------------------------------------|
| `enc_in_policy`  | `int` (0-2) | `1`     | Incoming encryption policy. `0`=disabled, `1`=enabled, `2`=forced. |
| `enc_out_policy` | `int` (0-2) | `1`     | Outgoing encryption policy. Same scale.                            |
| `enc_level`      | `int` (0-2) | `2`     | Encryption level. `0`=plaintext, `1`=rc4, `2`=both.                |

### Bandwidth / connections (global)

| Key                              | Type    | Default | Description                                             |
|----------------------------------|---------|---------|---------------------------------------------------------|
| `max_connections_global`         | `int`   | `200`   | Global max connections.                                 |
| `max_upload_speed`               | `float` | `-1.0`  | Global max upload speed in KiB/s; `-1.0` = unlimited.   |
| `max_download_speed`             | `float` | `-1.0`  | Global max download speed in KiB/s; `-1.0` = unlimited. |
| `max_upload_slots_global`        | `int`   | `4`     | Global max upload slots.                                |
| `max_half_open_connections`      | `int`   | `50`    | Max half-open connections (Windows: lower).             |
| `max_connections_per_second`     | `int`   | `20`    | Max new connections per second.                         |
| `ignore_limits_on_local_network` | `bool`  | `True`  | Whether to ignore rate limits for local network peers.  |
| `rate_limit_ip_overhead`         | `bool`  | `True`  | Whether to include IP overhead in rate limits.          |

### Bandwidth / connections (per-torrent)

| Key                              | Type  | Default | Description                                                |
|----------------------------------|-------|---------|------------------------------------------------------------|
| `max_connections_per_torrent`    | `int` | `-1`    | Per-torrent max connections; `-1` = unlimited.             |
| `max_upload_slots_per_torrent`   | `int` | `-1`    | Per-torrent max upload slots; `-1` = unlimited.            |
| `max_upload_speed_per_torrent`   | `int` | `-1`    | Per-torrent max upload speed in KiB/s; `-1` = unlimited.   |
| `max_download_speed_per_torrent` | `int` | `-1`    | Per-torrent max download speed in KiB/s; `-1` = unlimited. |

### Queue / active management

| Key                        | Type   | Default | Description                                            |
|----------------------------|--------|---------|--------------------------------------------------------|
| `max_active_seeding`       | `int`  | `5`     | Max active seeding torrents.                           |
| `max_active_downloading`   | `int`  | `3`     | Max active downloading torrents.                       |
| `max_active_limit`         | `int`  | `8`     | Max total active torrents.                             |
| `dont_count_slow_torrents` | `bool` | `False` | Whether to exclude slow torrents from active limits.   |
| `queue_new_to_top`         | `bool` | `False` | Whether to place new torrents at the top of the queue. |
| `auto_managed`             | `bool` | `True`  | Whether new torrents are auto-managed by the queue.    |
| `auto_manage_prefer_seeds` | `bool` | `False` | Whether auto-manage prefers seeds over downloads.      |

### Seeding / ratio

| Key                     | Type            | Default | Description                                      |
|-------------------------|-----------------|---------|--------------------------------------------------|
| `stop_seed_at_ratio`    | `bool`          | `False` | Whether to stop seeding at `stop_seed_ratio`.    |
| `remove_seed_at_ratio`  | `bool`          | `False` | Whether to remove torrents at `stop_seed_ratio`. |
| `stop_seed_ratio`       | `float`         | `2.00`  | Ratio at which to stop seeding.                  |
| `share_ratio_limit`     | `float`         | `2.00`  | Share ratio limit.                               |
| `seed_time_ratio_limit` | `float`         | `7.00`  | Seed time ratio limit.                           |
| `seed_time_limit`       | `int` (minutes) | `180`   | Seed time limit in minutes.                      |

### Torrent options (defaults for new torrents)

| Key                            | Type   | Default | Description                         |
|--------------------------------|--------|---------|-------------------------------------|
| `prioritize_first_last_pieces` | `bool` | `False` | Default for new torrents.           |
| `sequential_download`          | `bool` | `False` | Default for new torrents.           |
| `add_paused`                   | `bool` | `False` | Whether to add new torrents paused. |
| `super_seeding`                | `bool` | `False` | Default for new torrents.           |
| `shared`                       | `bool` | `False` | Default for new torrents.           |

### Plugins

| Key               | Type        | Default | Description               |
|-------------------|-------------|---------|---------------------------|
| `enabled_plugins` | `List[str]` | `[]`    | Names of enabled plugins. |

### Path chooser UI

| Key                                             | Type   | Default | Description |
|-------------------------------------------------|--------|---------|-------------|
| `path_chooser_show_chooser_button_on_localhost` | `bool` | `True`  | UI config.  |
| `path_chooser_auto_complete_enabled`            | `bool` | `True`  | UI config.  |
| `path_chooser_accelerator_string`               | `str`  | `'Tab'` | UI config.  |
| `path_chooser_max_popup_rows`                   | `int`  | `20`    | UI config.  |
| `path_chooser_show_hidden_files`                | `bool` | `False` | UI config.  |

### Updates

| Key                 | Type   | Default | Description                               |
|---------------------|--------|---------|-------------------------------------------|
| `new_release_check` | `bool` | `True`  | Whether to check for new Deluge releases. |

### Proxy (nested dict)

| Key                         | Type        | Default | Description                                                                                      |
|-----------------------------|-------------|---------|--------------------------------------------------------------------------------------------------|
| `type`                      | `int` (0-6) | `0`     | Proxy type. `0`=None, `1`=Socks4, `2`=Socks5, `3`=Socks5+Auth, `4`=HTTP, `5`=HTTP+Auth, `6`=I2P. |
| `hostname`                  | `str`       | `''`    | Proxy server hostname.                                                                           |
| `username`                  | `str`       | `''`    | Proxy username.                                                                                  |
| `password`                  | `str`       | `''`    | Proxy password.                                                                                  |
| `port`                      | `int`       | `8080`  | Proxy port.                                                                                      |
| `proxy_hostnames`           | `bool`      | `True`  | Whether to proxy hostname lookups.                                                               |
| `proxy_peer_connections`    | `bool`      | `True`  | Whether to proxy peer connections.                                                               |
| `proxy_tracker_connections` | `bool`      | `True`  | Whether to proxy tracker connections.                                                            |
| `force_proxy`               | `bool`      | `False` | Whether to force proxy usage.                                                                    |
| `anonymous_mode`            | `bool`      | `False` | Whether to enable anonymous mode.                                                                |

### Miscellaneous

| Key                 | Type            | Default                      | Description                      |
|---------------------|-----------------|------------------------------|----------------------------------|
| `peer_tos`          | `str` (hex)     | `'0x00'`                     | Type of service field for peers. |
| `geoip_db_location` | `str`           | `/usr/share/GeoIP/GeoIP.dat` | Path to GeoIP database.          |
| `cache_size`        | `int`           | `512`                        | Disk cache size in 16KB blocks.  |
| `cache_expiry`      | `int` (seconds) | `60`                         | Disk cache expiry time.          |

**Total**: 57 top-level keys (including the nested `proxy` dict with 10 sub-keys).

---

## Proxy return dict

Returned by `core.get_proxy()`. Reads from **live libtorrent session settings**, NOT from the config. 8 keys (config's
`proxy` sub-dict has 10 - `force_proxy` and `anonymous_mode` are NOT included here).

| Key                         | Type        | Description                                                     |
|-----------------------------|-------------|-----------------------------------------------------------------|
| `type`                      | `int` (0-6) | Proxy type. Same enum as config's `proxy.type`.                 |
| `hostname`                  | `str`       | Proxy hostname. For I2P (`type==6`), reads from `i2p_hostname`. |
| `username`                  | `str`       | Proxy username.                                                 |
| `password`                  | `str`       | Proxy password.                                                 |
| `port`                      | `int`       | Proxy port. For I2P (`type==6`), reads from `i2p_port`.         |
| `proxy_hostnames`           | `bool`      | Whether hostname lookups are proxied.                           |
| `proxy_peer_connections`    | `bool`      | Whether peer connections are proxied.                           |
| `proxy_tracker_connections` | `bool`      | Whether tracker connections are proxied.                        |

---

## Account dict

Each element of `core.get_known_accounts()` return list.

| Key             | Type  | Description                                                           |
|-----------------|-------|-----------------------------------------------------------------------|
| `username`      | `str` | Account username.                                                     |
| `password`      | `str` | Password hash (or plaintext for `localclient`).                       |
| `authlevel`     | `str` | Auth level name: `NONE`, `READONLY`, `DEFAULT`, `NORMAL`, or `ADMIN`. |
| `authlevel_int` | `int` | Auth level integer: `0`, `1`, `5`, or `10`.                           |

---

## Completion paths

Returned by `core.get_completion_paths(args)`.

**Input `args` dict**:

| Key                 | Type   | Description                           |
|---------------------|--------|---------------------------------------|
| `completion_text`   | `str`  | Path prefix to complete.              |
| `show_hidden_files` | `bool` | Whether to include hidden files/dirs. |

**Return dict**: same `args` dict with an added key:

| Key     | Type        | Description                                                         |
|---------|-------------|---------------------------------------------------------------------|
| `paths` | `List[str]` | Sorted matching directory paths (with trailing `/`), empty if none. |

All original input keys are passed through unchanged.

---

## Session status keys

Returned by `core.get_session_status(keys)`. `keys` is required — pass `[]` to return the full dict.

Three key sources: libtorrent session metrics (all `int`), rate keys (all `float`, bytes/sec), and cache hit ratios (all
`float`, 0.0–1.0).

### Rate keys - `float` (bytes/sec)

Computed over a 2-second interval from the corresponding `net.*` counter.

| Key                         | Description                                         |
|-----------------------------|-----------------------------------------------------|
| `download_rate`             | Total download rate.                                |
| `upload_rate`               | Total upload rate.                                  |
| `payload_download_rate`     | Payload download rate (excludes protocol overhead). |
| `payload_upload_rate`       | Payload upload rate.                                |
| `ip_overhead_download_rate` | IP overhead download rate.                          |
| `ip_overhead_upload_rate`   | IP overhead upload rate.                            |
| `tracker_download_rate`     | Tracker response download rate.                     |
| `tracker_upload_rate`       | Tracker request upload rate.                        |
| `dht_download_rate`         | DHT download rate.                                  |
| `dht_upload_rate`           | DHT upload rate.                                    |

### Cache hit ratios - `float` (0.0–1.0)

| Key               | Description                 |
|-------------------|-----------------------------|
| `write_hit_ratio` | Disk write cache hit ratio. |
| `read_hit_ratio`  | Disk read cache hit ratio.  |

### libtorrent session metrics - `int`

> **Note**: The exact set of available metrics depends on the libtorrent version the daemon was built against. Clients
> should treat unknown metric keys as `0` or missing rather than erroring.

#### `peer.*` - peer connection counters

| Key                                     | Description                                   |
|-----------------------------------------|-----------------------------------------------|
| `peer.error_peers`                      | Peers disconnected due to error.              |
| `peer.disconnected_peers`               | Total disconnected peers.                     |
| `peer.eof_peers`                        | Peers disconnected due to EOF.                |
| `peer.connreset_peers`                  | Peers disconnected due to connection reset.   |
| `peer.connrefused_peers`                | Peers disconnected due to connection refused. |
| `peer.connaborted_peers`                | Peers disconnected due to connection aborted. |
| `peer.notconnected_peers`               | Peers never connected.                        |
| `peer.perm_peers`                       | Peers disconnected due to permanent error.    |
| `peer.buffer_peers`                     | Peers disconnected due to buffer error.       |
| `peer.unreachable_peers`                | Peers unreachable.                            |
| `peer.broken_pipe_peers`                | Peers disconnected due to broken pipe.        |
| `peer.addrinuse_peers`                  | Peers disconnected due to address in use.     |
| `peer.no_access_peers`                  | Peers disconnected due to no access.          |
| `peer.invalid_arg_peers`                | Peers disconnected due to invalid argument.   |
| `peer.aborted_peers`                    | Peers aborted.                                |
| `peer.piece_requests`                   | Total piece requests sent.                    |
| `peer.max_piece_requests`               | Max piece requests in flight.                 |
| `peer.invalid_piece_requests`           | Invalid piece requests received.              |
| `peer.choked_piece_requests`            | Choked piece requests.                        |
| `peer.cancelled_piece_requests`         | Cancelled piece requests.                     |
| `peer.piece_rejects`                    | Piece requests rejected.                      |
| `peer.error_incoming_peers`             | Incoming peers with errors.                   |
| `peer.error_outgoing_peers`             | Outgoing peers with errors.                   |
| `peer.error_rc4_peers`                  | Peers with RC4 errors.                        |
| `peer.error_encrypted_peers`            | Peers with encryption errors.                 |
| `peer.error_tcp_peers`                  | TCP peers with errors.                        |
| `peer.error_utp_peers`                  | uTP peers with errors.                        |
| `peer.connect_timeouts`                 | Peer connection timeouts.                     |
| `peer.uninteresting_peers`              | Peers marked uninteresting.                   |
| `peer.timeout_peers`                    | Peers timed out.                              |
| `peer.no_memory_peers`                  | Peers disconnected due to no memory.          |
| `peer.too_many_peers`                   | Peers rejected due to too many.               |
| `peer.transport_timeout_peers`          | Peers with transport timeout.                 |
| `peer.num_banned_peers`                 | Banned peers.                                 |
| `peer.banned_for_hash_failure`          | Peers banned for hash failure.                |
| `peer.connection_attempts`              | Total connection attempts.                    |
| `peer.connection_attempt_loops`         | Connection attempt loops.                     |
| `peer.boost_connection_attempts`        | Boosted connection attempts.                  |
| `peer.missed_connection_attempts`       | Missed connection attempts.                   |
| `peer.no_peer_connection_attempts`      | No-peer connection attempts.                  |
| `peer.incoming_connections`             | Incoming connections received.                |
| `peer.num_tcp_peers`                    | Current TCP peers.                            |
| `peer.num_socks5_peers`                 | Current Socks5 peers.                         |
| `peer.num_http_proxy_peers`             | Current HTTP proxy peers.                     |
| `peer.num_utp_peers`                    | Current uTP peers.                            |
| `peer.num_i2p_peers`                    | Current I2P peers.                            |
| `peer.num_ssl_peers`                    | Current SSL peers.                            |
| `peer.num_ssl_socks5_peers`             | Current SSL Socks5 peers.                     |
| `peer.num_ssl_http_proxy_peers`         | Current SSL HTTP proxy peers.                 |
| `peer.num_ssl_utp_peers`                | Current SSL uTP peers.                        |
| `peer.num_peers_half_open`              | Half-open peers.                              |
| `peer.num_peers_connected`              | Connected peers.                              |
| `peer.num_peers_up_interested`          | Peers interested in uploading to us.          |
| `peer.num_peers_down_interested`        | Peers interested in downloading from us.      |
| `peer.num_peers_up_unchoked_all`        | All unchoked upload peers.                    |
| `peer.num_peers_up_unchoked_optimistic` | Optimistic unchoked upload peers.             |
| `peer.num_peers_up_unchoked`            | Unchoked upload peers.                        |
| `peer.num_peers_down_unchoked`          | Unchoked download peers.                      |
| `peer.num_peers_up_requests`            | Peers with active upload requests.            |
| `peer.num_peers_down_requests`          | Peers with active download requests.          |
| `peer.num_peers_end_game`               | Peers in end-game mode.                       |
| `peer.num_peers_up_disk`                | Peers with active disk writes.                |
| `peer.num_peers_down_disk`              | Peers with active disk reads.                 |

#### `net.*` - network counters

| Key                            | Description                                                |
|--------------------------------|------------------------------------------------------------|
| `net.on_read_counter`          | Read event count.                                          |
| `net.on_write_counter`         | Write event count.                                         |
| `net.on_tick_counter`          | Tick event count.                                          |
| `net.on_lsd_counter`           | LSD event count.                                           |
| `net.on_lsd_peer_counter`      | LSD peer event count.                                      |
| `net.on_udp_counter`           | UDP event count.                                           |
| `net.on_accept_counter`        | Accept event count.                                        |
| `net.on_disk_queue_counter`    | Disk queue event count.                                    |
| `net.on_disk_counter`          | Disk event count.                                          |
| `net.sent_payload_bytes`       | Total payload bytes sent.                                  |
| `net.sent_bytes`               | Total bytes sent (including overhead).                     |
| `net.sent_ip_overhead_bytes`   | Total IP overhead bytes sent.                              |
| `net.sent_tracker_bytes`       | Total bytes sent to trackers.                              |
| `net.recv_payload_bytes`       | Total payload bytes received.                              |
| `net.recv_bytes`               | Total bytes received (including overhead).                 |
| `net.recv_ip_overhead_bytes`   | Total IP overhead bytes received.                          |
| `net.recv_tracker_bytes`       | Total bytes received from trackers.                        |
| `net.limiter_up_queue`         | Upload limiter queue length.                               |
| `net.limiter_down_queue`       | Download limiter queue length.                             |
| `net.limiter_up_bytes`         | Upload limiter queue bytes.                                |
| `net.limiter_down_bytes`       | Download limiter queue bytes.                              |
| `net.recv_failed_bytes`        | Bytes received but failed.                                 |
| `net.recv_redundant_bytes`     | Redundant bytes received.                                  |
| `net.has_incoming_connections` | Whether any incoming connections have been received (0/1). |

#### `ses.*` - session/torrent counters

| Key                                | Description                            |
|------------------------------------|----------------------------------------|
| `ses.num_checking_torrents`        | Torrents currently being checked.      |
| `ses.num_stopped_torrents`         | Stopped torrents.                      |
| `ses.num_upload_only_torrents`     | Upload-only torrents.                  |
| `ses.num_downloading_torrents`     | Downloading torrents.                  |
| `ses.num_seeding_torrents`         | Seeding torrents.                      |
| `ses.num_queued_seeding_torrents`  | Queued seeding torrents.               |
| `ses.num_queued_download_torrents` | Queued downloading torrents.           |
| `ses.num_error_torrents`           | Torrents in error state.               |
| `ses.non_filter_torrents`          | Torrents not filtered.                 |
| `ses.num_piece_passed`             | Pieces that passed hash check.         |
| `ses.num_piece_failed`             | Pieces that failed hash check.         |
| `ses.num_have_pieces`              | Total pieces have across all torrents. |
| `ses.num_total_pieces_added`       | Total pieces added.                    |
| `ses.num_unchoke_slots`            | Unchoke slots.                         |

#### `disk.*` - disk I/O counters

| Key                                | Description                          |
|------------------------------------|--------------------------------------|
| `disk.num_blocks_read`             | Disk blocks read.                    |
| `disk.num_blocks_written`          | Disk blocks written.                 |
| `disk.num_write_ops`               | Disk write operations.               |
| `disk.num_read_ops`                | Disk read operations.                |
| `disk.num_fenced_delete_files`     | Pending delete file operations.      |
| `disk.num_fenced_check_fastresume` | Pending fastresume check operations. |
| `disk.num_fenced_save_resume_data` | Pending save resume data operations. |
| `disk.num_fenced_rename_file`      | Pending rename file operations.      |
| `disk.num_fenced_stop_torrent`     | Pending stop torrent operations.     |
| `disk.num_fenced_flush_piece`      | Pending flush piece operations.      |
| `disk.num_fenced_flush_hashed`     | Pending flush hashed operations.     |
| `disk.num_fenced_flush_storage`    | Pending flush storage operations.    |
| `disk.num_fenced_file_priority`    | Pending file priority operations.    |
| `disk.num_fenced_load_torrent`     | Pending load torrent operations.     |
| `disk.num_fenced_clear_piece`      | Pending clear piece operations.      |
| `disk.num_fenced_tick_storage`     | Pending tick storage operations.     |

#### `dht.*` - DHT counters

| Key                                 | Description                             |
|-------------------------------------|-----------------------------------------|
| `dht.dht_nodes`                     | DHT nodes.                              |
| `dht.dht_node_cache`                | DHT node cache.                         |
| `dht.dht_torrents`                  | DHT torrents.                           |
| `dht.dht_peers`                     | DHT peers.                              |
| `dht.dht_immutable_data`            | DHT immutable data items.               |
| `dht.dht_mutable_data`              | DHT mutable data items.                 |
| `dht.dht_allocated_observers`       | DHT allocated observers.                |
| `dht.dht_messages_in`               | DHT messages received.                  |
| `dht.dht_messages_out`              | DHT messages sent.                      |
| `dht.dht_messages_in_dropped`       | DHT messages dropped (in).              |
| `dht.dht_messages_out_dropped`      | DHT messages dropped (out).             |
| `dht.dht_bytes_in`                  | DHT bytes received.                     |
| `dht.dht_bytes_out`                 | DHT bytes sent.                         |
| `dht.dht_ping_in`                   | DHT pings received.                     |
| `dht.dht_ping_out`                  | DHT pings sent.                         |
| `dht.dht_find_node_in`              | DHT find_node received.                 |
| `dht.dht_find_node_out`             | DHT find_node sent.                     |
| `dht.dht_get_peers_in`              | DHT get_peers received.                 |
| `dht.dht_get_peers_out`             | DHT get_peers sent.                     |
| `dht.dht_announce_peer_in`          | DHT announce_peer received.             |
| `dht.dht_announce_peer_out`         | DHT announce_peer sent.                 |
| `dht.dht_get_in`                    | DHT get received.                       |
| `dht.dht_get_out`                   | DHT get sent.                           |
| `dht.dht_put_in`                    | DHT put received.                       |
| `dht.dht_put_out`                   | DHT put sent.                           |
| `dht.dht_sample_infohashes_in`      | DHT sample_infohashes received.         |
| `dht.dht_sample_infohashes_out`     | DHT sample_infohashes sent.             |
| `dht.dht_invalid_announce`          | Invalid DHT announce messages.          |
| `dht.dht_invalid_get_peers`         | Invalid DHT get_peers messages.         |
| `dht.dht_invalid_find_node`         | Invalid DHT find_node messages.         |
| `dht.dht_invalid_put`               | Invalid DHT put messages.               |
| `dht.dht_invalid_get`               | Invalid DHT get messages.               |
| `dht.dht_invalid_sample_infohashes` | Invalid DHT sample_infohashes messages. |

#### `utp.*` - uTP counters

| Key                            | Description                     |
|--------------------------------|---------------------------------|
| `utp.utp_packet_loss`          | uTP packet loss count.          |
| `utp.utp_timeout`              | uTP timeout count.              |
| `utp.utp_packets_in`           | uTP packets received.           |
| `utp.utp_packets_out`          | uTP packets sent.               |
| `utp.utp_fast_retransmit`      | uTP fast retransmit count.      |
| `utp.utp_packet_resend`        | uTP packet resend count.        |
| `utp.utp_samples_above_target` | uTP samples above target delay. |
| `utp.utp_samples_below_target` | uTP samples below target delay. |
| `utp.utp_payload_pkts_in`      | uTP payload packets received.   |
| `utp.utp_payload_pkts_out`     | uTP payload packets sent.       |
| `utp.utp_invalid_pkts_in`      | Invalid uTP packets received.   |
| `utp.utp_redundant_pkts_in`    | Redundant uTP packets received. |
| `utp.num_utp_idle`             | uTP sockets idle.               |
| `utp.num_utp_syn_sent`         | uTP sockets with SYN sent.      |
| `utp.num_utp_connected`        | uTP sockets connected.          |
| `utp.num_utp_fin_sent`         | uTP sockets with FIN sent.      |
| `utp.num_utp_close_wait`       | uTP sockets in close wait.      |
| `utp.num_utp_deleted`          | uTP sockets deleted.            |

#### `sock_bufs.*` - socket buffer size histograms

Counts of sockets with send/recv buffer sizes in each bucket (3–20, in 1KB increments).

| Key                            | Description                     |
|--------------------------------|---------------------------------|
| `sock_bufs.socket_send_size3`  | Sockets with send buffer ~3KB.  |
| `sock_bufs.socket_send_size4`  | Sockets with send buffer ~4KB.  |
| `sock_bufs.socket_send_size5`  | Sockets with send buffer ~5KB.  |
| `sock_bufs.socket_send_size6`  | Sockets with send buffer ~6KB.  |
| `sock_bufs.socket_send_size7`  | Sockets with send buffer ~7KB.  |
| `sock_bufs.socket_send_size8`  | Sockets with send buffer ~8KB.  |
| `sock_bufs.socket_send_size9`  | Sockets with send buffer ~9KB.  |
| `sock_bufs.socket_send_size10` | Sockets with send buffer ~10KB. |
| `sock_bufs.socket_send_size11` | Sockets with send buffer ~11KB. |
| `sock_bufs.socket_send_size12` | Sockets with send buffer ~12KB. |
| `sock_bufs.socket_send_size13` | Sockets with send buffer ~13KB. |
| `sock_bufs.socket_send_size14` | Sockets with send buffer ~14KB. |
| `sock_bufs.socket_send_size15` | Sockets with send buffer ~15KB. |
| `sock_bufs.socket_send_size16` | Sockets with send buffer ~16KB. |
| `sock_bufs.socket_send_size17` | Sockets with send buffer ~17KB. |
| `sock_bufs.socket_send_size18` | Sockets with send buffer ~18KB. |
| `sock_bufs.socket_send_size19` | Sockets with send buffer ~19KB. |
| `sock_bufs.socket_send_size20` | Sockets with send buffer ~20KB. |
| `sock_bufs.socket_recv_size3`  | Sockets with recv buffer ~3KB.  |
| `sock_bufs.socket_recv_size4`  | Sockets with recv buffer ~4KB.  |
| `sock_bufs.socket_recv_size5`  | Sockets with recv buffer ~5KB.  |
| `sock_bufs.socket_recv_size6`  | Sockets with recv buffer ~6KB.  |
| `sock_bufs.socket_recv_size7`  | Sockets with recv buffer ~7KB.  |
| `sock_bufs.socket_recv_size8`  | Sockets with recv buffer ~8KB.  |
| `sock_bufs.socket_recv_size9`  | Sockets with recv buffer ~9KB.  |
| `sock_bufs.socket_recv_size10` | Sockets with recv buffer ~10KB. |
| `sock_bufs.socket_recv_size11` | Sockets with recv buffer ~11KB. |
| `sock_bufs.socket_recv_size12` | Sockets with recv buffer ~12KB. |
| `sock_bufs.socket_recv_size13` | Sockets with recv buffer ~13KB. |
| `sock_bufs.socket_recv_size14` | Sockets with recv buffer ~14KB. |
| `sock_bufs.socket_recv_size15` | Sockets with recv buffer ~15KB. |
| `sock_bufs.socket_recv_size16` | Sockets with recv buffer ~16KB. |
| `sock_bufs.socket_recv_size17` | Sockets with recv buffer ~17KB. |
| `sock_bufs.socket_recv_size18` | Sockets with recv buffer ~18KB. |
| `sock_bufs.socket_recv_size19` | Sockets with recv buffer ~19KB. |
| `sock_bufs.socket_recv_size20` | Sockets with recv buffer ~20KB. |

#### `tracker.*` - tracker counters

| Key                                    | Description               |
|----------------------------------------|---------------------------|
| `tracker.num_queued_tracker_announces` | Queued tracker announces. |
