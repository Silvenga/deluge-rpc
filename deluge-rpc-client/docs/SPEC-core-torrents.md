# SPEC - `core.*` Torrent Methods

`deluge-torrent/deluge` @ commit `e58075416dedd53636e89b1cd240f86f2e7c2ee0`.

## Methods

### Add / remove

| Method                          | Args                                                                     | Returns                                | Auth   | Description                                                                                                                                                                                                            |
|---------------------------------|--------------------------------------------------------------------------|----------------------------------------|--------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `core.add_torrent_file`         | `(filename: str, filedump: Union[str, bytes], options: dict)`            | `Optional[str]` (torrent_id hex)       | NORMAL | Adds a single torrent file to the session. `filedump` is base64-encoded torrent data. `options` configures the torrent (see "Torrent options keys"). Returns the torrent_id on success, `None` on failure.             |
| `core.add_torrent_file_async`   | `(filename: str, filedump: str, options: dict, save_state: bool = True)` | `Deferred[Optional[str]]` (torrent_id) | NORMAL | Async variant of `add_torrent_file`. `save_state=True` persists the session state after adding.                                                                                                                        |
| `core.add_torrent_files`        | `(torrent_files: List[Tuple[str, Union[str, bytes], dict]])`             | `Deferred[List[AddTorrentError]]`      | NORMAL | Adds multiple torrents at once. Each tuple is `(filename, filedump, options)`. The returned list contains only errors for torrents that failed; empty list = all succeeded. Each error serializes to a string message. |
| `core.add_torrent_url`          | `(url: str, options: dict, headers: dict = None)`                        | `Deferred[Optional[str]]` (torrent_id) | NORMAL | Downloads a torrent file from `url`, then adds it to the session. `headers` are HTTP request headers. Returns torrent_id or `None` on failure.                                                                         |
| `core.add_torrent_magnet`       | `(uri: str, options: dict)`                                              | `str` (torrent_id hex)                 | NORMAL | Adds a torrent from a magnet URI. Returns the torrent_id.                                                                                                                                                              |
| `core.prefetch_magnet_metadata` | `(magnet: str, timeout: int = 30)`                                       | `Deferred[Tuple[str, bytes]]`          | NORMAL | Downloads magnet metadata without adding the torrent to the session. Returns `(torrent_id, metadata)` where `metadata` is bencoded torrent data. `timeout` is seconds to wait for metadata.                            |
| `core.remove_torrent`           | `(torrent_id: str, remove_data: bool)`                                   | `bool`                                 | NORMAL | Removes a single torrent. `remove_data=True` also deletes downloaded files. Returns `True` on success. Raises `InvalidTorrentError` if torrent_id not found.                                                           |
| `core.remove_torrents`          | `(torrent_ids: List[str], remove_data: bool)`                            | `Deferred[List[Tuple[str, str]]]`      | NORMAL | Removes multiple torrents. Each tuple in the return list is `(torrent_id, error_message)` for failed removals; empty list = all succeeded.                                                                             |

### State control

| Method                      | Args                                                                                               | Returns | Auth   | Description                                                                                                     |
|-----------------------------|----------------------------------------------------------------------------------------------------|---------|--------|-----------------------------------------------------------------------------------------------------------------|
| `core.pause_torrent`        | `(torrent_id: str)`                                                                                | `None`  | NORMAL | Pauses a single torrent.                                                                                        |
| `core.pause_torrents`       | `(torrent_ids: List[str] = None)`                                                                  | `None`  | NORMAL | Pauses multiple torrents. `None` pauses all.                                                                    |
| `core.resume_torrent`       | `(torrent_id: str)`                                                                                | `None`  | NORMAL | Resumes a single paused torrent.                                                                                |
| `core.resume_torrents`      | `(torrent_ids: List[str] = None)`                                                                  | `None`  | NORMAL | Resumes multiple torrents. `None` resumes all.                                                                  |
| `core.force_reannounce`     | `(torrent_ids: List[str])`                                                                         | `None`  | NORMAL | Forces tracker reannounce for the given torrents.                                                               |
| `core.force_recheck`        | `(torrent_ids: List[str])`                                                                         | `None`  | NORMAL | Forces a data recheck (hash check) for the given torrents.                                                      |
| `core.set_torrent_options`  | `(torrent_ids: List[str], options: Dict[str, Any])`                                                | `None`  | NORMAL | Sets per-torrent options. `options` keys are listed in "Torrent options keys".                                  |
| `core.connect_peer`         | `(torrent_id: str, ip: str, port: int)`                                                            | `None`  | NORMAL | Manually connects to a peer for the given torrent.                                                              |
| `core.move_storage`         | `(torrent_ids: List[str], dest: str)`                                                              | `None`  | NORMAL | Moves downloaded data to a new location for the given torrents.                                                 |
| `core.set_ssl_torrent_cert` | `(torrent_id: str, certificate: str, private_key: str, dh_params: str, save_to_disk: bool = True)` | `None`  | NORMAL | Sets SSL certificates for connecting to SSL peers of the given torrent. `save_to_disk=True` persists the certs. |

### Queries

| Method                     | Args                                                        | Returns                                | Auth   | Description                                                                                                                                                                                                  |
|----------------------------|-------------------------------------------------------------|----------------------------------------|--------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `core.get_torrent_status`  | `(torrent_id: str, keys: List[str], diff: bool = False)`    | `dict` (`{key: value}`)                | NORMAL | Gets status values for a single torrent. `keys=[]` returns ALL keys. See "Torrent status keys". `diff=True` returns only keys changed since the last call in the same session.                               |
| `core.get_torrents_status` | `(filter_dict: dict, keys: List[str], diff: bool = False)`  | `dict` (`{torrent_id: {key: value}}`)  | NORMAL | Gets status values for multiple torrents, optionally filtered. `filter_dict={}` returns all. `keys=[]` returns ALL keys. See "Torrent status keys" and "Filter dict". `diff=True` returns only changed keys. |
| `core.get_filter_tree`     | `(show_zero_hits: bool = True, hide_cat: List[str] = None)` | `Dict[str, List[Tuple[str, int]]]`     | NORMAL | Returns `{field: [(value, count)]}` for use in sidebar UIs. See "Filter tree fields". `show_zero_hits=False` removes zero-count entries. `hide_cat` excludes listed fields.                                  |
| `core.get_session_state`   | `()`                                                        | `List[str]` (torrent_ids, 40-char hex) | NORMAL | Returns all torrent_ids in the session. Filtered by auth level: non-admin users see only owned/shared torrents.                                                                                              |
| `core.get_magnet_uri`      | `(torrent_id: str)`                                         | `str`                                  | NORMAL | Returns the magnet URI for the torrent (`magnet:?xt=urn:btih:...`).                                                                                                                                          |
| `core.get_path_size`       | `(path)`                                                    | `int` (bytes; `-1` if inaccessible)    | NORMAL | Returns the size of a file or directory in bytes. `-1` if the path doesn't exist or is inaccessible.                                                                                                         |

### Trackers / files / folders

| Method                      | Args                                                  | Returns                      | Auth   | Description                                                                                                                 |
|-----------------------------|-------------------------------------------------------|------------------------------|--------|-----------------------------------------------------------------------------------------------------------------------------|
| `core.set_torrent_trackers` | `(torrent_id: str, trackers: List[Dict[str, Any]])`   | `None`                       | NORMAL | Replaces the tracker list for a torrent. Each element: `{"url": str, "tier": int}`.                                         |
| `core.rename_files`         | `(torrent_id: str, filenames: List[Tuple[int, str]])` | `Deferred` (resolves `None`) | NORMAL | Renames files within a torrent. Each tuple is `(file_index, new_filename)`. Watch `TorrentFileRenamedEvent` for completion. |
| `core.rename_folder`        | `(torrent_id: str, folder: str, new_folder: str)`     | `Deferred` (resolves `None`) | NORMAL | Renames a folder within a torrent. Watch `TorrentFolderRenamedEvent` for completion.                                        |

### Queue

| Method              | Args                       | Returns | Auth   | Description                                                               |
|---------------------|----------------------------|---------|--------|---------------------------------------------------------------------------|
| `core.queue_top`    | `(torrent_ids: List[str])` | `None`  | NORMAL | Moves torrents to the top of the queue. Emits `TorrentQueueChangedEvent`. |
| `core.queue_up`     | `(torrent_ids: List[str])` | `None`  | NORMAL | Moves torrents up one position in the queue.                              |
| `core.queue_down`   | `(torrent_ids: List[str])` | `None`  | NORMAL | Moves torrents down one position in the queue.                            |
| `core.queue_bottom` | `(torrent_ids: List[str])` | `None`  | NORMAL | Moves torrents to the bottom of the queue.                                |

---

## Torrent status keys

Valid `keys` for `core.get_torrent_status` and `core.get_torrents_status`. Pass `keys=[]` to fetch all.

### Time / transfer stats

| Key                      | Type  | Description                                                        |
|--------------------------|-------|--------------------------------------------------------------------|
| `active_time`            | `int` | Seconds since the torrent was added.                               |
| `all_time_download`      | `int` | Total bytes downloaded (all-time).                                 |
| `completed_time`         | `int` | Unix timestamp when download completed; `-1` if not completed.     |
| `finished_time`          | `int` | Seconds the torrent has been in finished state.                    |
| `last_seen_complete`     | `int` | Unix timestamp when torrent was last seen complete; `-1` if never. |
| `seeding_time`           | `int` | Seconds spent seeding.                                             |
| `time_added`             | `int` | Unix timestamp when the torrent was added.                         |
| `time_since_download`    | `int` | Seconds since last download activity; `-1` if never.               |
| `time_since_transfer`    | `int` | Seconds since last upload or download; `-1` if never.              |
| `time_since_upload`      | `int` | Seconds since last upload activity; `-1` if never.                 |
| `total_done`             | `int` | Bytes downloaded and verified (pieces).                            |
| `total_payload_download` | `int` | Payload bytes downloaded (excludes protocol overhead).             |
| `total_payload_upload`   | `int` | Payload bytes uploaded (excludes protocol overhead).               |
| `total_remaining`        | `int` | Bytes remaining to download.                                       |
| `total_uploaded`         | `int` | Total bytes uploaded (all-time).                                   |
| `total_wanted`           | `int` | Bytes of files marked for download (excludes skipped files).       |

### Rates

| Key                     | Type  | Description                                                    |
|-------------------------|-------|----------------------------------------------------------------|
| `download_payload_rate` | `int` | Current payload download speed in bytes/sec.                   |
| `upload_payload_rate`   | `int` | Current payload upload speed in bytes/sec.                     |
| `eta`                   | `int` | Estimated seconds to completion; `-1` if >1 year, `0` if idle. |

### Ratios / seed metrics

| Key                  | Type    | Description                                                |
|----------------------|---------|------------------------------------------------------------|
| `distributed_copies` | `float` | Distributed copies (swarm availability); `>= 0.0`.         |
| `ratio`              | `float` | Share ratio; `-1.0` means infinity (when `total_done==0`). |
| `seed_rank`          | `int`   | Seed rank score used for queue ordering.                   |
| `seeds_peers_ratio`  | `float` | Seeds-to-peers ratio; `-1.0` if no incomplete peers.       |

### Peers / seeds

| Key           | Type         | Description                                       |
|---------------|--------------|---------------------------------------------------|
| `num_peers`   | `int`        | Connected peers (excludes seeds).                 |
| `num_seeds`   | `int`        | Connected seeds.                                  |
| `total_peers` | `int`        | Total peers in swarm (unconnected, from tracker). |
| `total_seeds` | `int`        | Total seeds in swarm (unconnected, from tracker). |
| `peers`       | `List[Dict]` | Connected peers. See "peers sub-dict".            |

### State

| Key             | Type    | Description                                                                                                       |
|-----------------|---------|-------------------------------------------------------------------------------------------------------------------|
| `state`         | `str`   | Current state. One of: `Allocating`, `Checking`, `Downloading`, `Seeding`, `Paused`, `Error`, `Queued`, `Moving`. |
| `paused`        | `bool`  | Whether the torrent is paused.                                                                                    |
| `progress`      | `float` | Download progress as `0.0`–`100.0`.                                                                               |
| `is_seed`       | `bool`  | Whether the torrent is seeding (download complete).                                                               |
| `is_finished`   | `bool`  | Whether the torrent has finished downloading.                                                                     |
| `seed_mode`     | `bool`  | Whether the torrent is in seed mode (started assuming all data present).                                          |
| `super_seeding` | `bool`  | Whether super seeding is enabled.                                                                                 |
| `message`       | `str`   | Status message; default `'OK'`.                                                                                   |
| `queue`         | `int`   | Queue position.                                                                                                   |
| `storage_mode`  | `str`   | Storage allocation mode: `'sparse'` or `'allocate'`.                                                              |

### Identity / metadata

| Key            | Type                  | Description                                                                                                          |
|----------------|-----------------------|----------------------------------------------------------------------------------------------------------------------|
| `hash`         | `str`                 | Info hash (same as torrent_id).                                                                                      |
| `name`         | `str`                 | Display name.                                                                                                        |
| `comment`      | `str`                 | Torrent comment; `''` if no metadata.                                                                                |
| `creator`      | `str`                 | Torrent creator; `''` if no metadata.                                                                                |
| `private`      | `bool`                | Whether the torrent is private; `False` if no metadata.                                                              |
| `num_files`    | `int`                 | Number of files; `0` if no metadata.                                                                                 |
| `num_pieces`   | `int`                 | Number of pieces; `0` if no metadata.                                                                                |
| `piece_length` | `int`                 | Piece length in bytes; `0` if no metadata.                                                                           |
| `total_size`   | `int`                 | Total size of all files in bytes; `0` if no metadata.                                                                |
| `files`        | `List[Dict]`          | File list. See "files sub-dict". `[]` if no metadata.                                                                |
| `orig_files`   | `List[Dict]`          | Original file paths (before rename). `[]` if no metadata.                                                            |
| `pieces`       | `List[int]` or `None` | Per-piece state. `None` if seeding or no metadata. Else: `0`=missing, `1`=available, `2`=downloading, `3`=completed. |

### File state

| Key               | Type          | Description                                                                                   |
|-------------------|---------------|-----------------------------------------------------------------------------------------------|
| `file_priorities` | `List[int]`   | Per-file priority: `0`=skip, `1`=low, `2`=normal, `5`=high, `7`=highest. `[]` if no metadata. |
| `file_progress`   | `List[float]` | Per-file progress `0.0`–`1.0`. `[]` if no metadata.                                           |

### Trackers

| Key              | Type         | Description                                                    |
|------------------|--------------|----------------------------------------------------------------|
| `tracker`        | `str`        | Current tracker URL.                                           |
| `tracker_host`   | `str`        | Short hostname of current tracker.                             |
| `tracker_status` | `str`        | Tracker status string, e.g. `'Announce OK'` or `'Error: ...'`. |
| `trackers`       | `List[Dict]` | Full tracker list. Each element: `{"url": str, "tier": int}`.  |

### Options (per-torrent, settable via `set_torrent_options`)

| Key                            | Type    | Description                                               |
|--------------------------------|---------|-----------------------------------------------------------|
| `auto_managed`                 | `bool`  | Whether the torrent is auto-managed by the queue.         |
| `is_auto_managed`              | `bool`  | Alias for `auto_managed`.                                 |
| `download_location`            | `str`   | Save path.                                                |
| `max_connections`              | `int`   | Max connections; `-1` = unlimited.                        |
| `max_download_speed`           | `float` | Max download speed in KiB/s; `-1.0` = unlimited.          |
| `max_upload_slots`             | `int`   | Max upload slots; `-1` = unlimited.                       |
| `max_upload_speed`             | `float` | Max upload speed in KiB/s; `-1.0` = unlimited.            |
| `move_completed`               | `bool`  | Whether to move when completed.                           |
| `move_completed_path`          | `str`   | Destination path for move-on-completion.                  |
| `owner`                        | `str`   | Username of the torrent owner.                            |
| `prioritize_first_last_pieces` | `bool`  | Whether to prioritize first and last pieces.              |
| `remove_at_ratio`              | `bool`  | Whether to remove the torrent when stop ratio is reached. |
| `sequential_download`          | `bool`  | Whether to download pieces sequentially.                  |
| `shared`                       | `bool`  | Whether the torrent is shared across users.               |
| `stop_at_ratio`                | `bool`  | Whether to stop at the stop ratio.                        |
| `stop_ratio`                   | `float` | The stop ratio.                                           |

### Sub-dicts

**`files` / `orig_files` element**:

```python
{"index": int, "path": str, "size": int, "offset": int}
```

**`peers` element**:

```python
{"client": str, "country": str, "down_speed": int, "ip": str, "progress": float, "seed": bool, "up_speed": int}
```

---

## Torrent options keys

Keys accepted by `core.set_torrent_options(torrent_ids, options)` and the `options` dict of `add_torrent_*` methods.

| Key                            | Type        | Default           | Description                                                              |
|--------------------------------|-------------|-------------------|--------------------------------------------------------------------------|
| `file_priorities`              | `List[int]` | `[]`              | Per-file priority: `0`=skip, `1`=low, `2`=normal, `5`=high, `7`=highest. |
| `max_connections`              | `int`       | `-1`              | Max connections; `-1` = unlimited.                                       |
| `max_download_speed`           | `float`     | `-1.0`            | Max download speed in KiB/s; `-1.0` = unlimited.                         |
| `max_upload_slots`             | `int`       | `-1`              | Max upload slots; `-1` = unlimited.                                      |
| `max_upload_speed`             | `float`     | `-1.0`            | Max upload speed in KiB/s; `-1.0` = unlimited.                           |
| `move_completed`               | `bool`      | `False`           | Whether to move when completed.                                          |
| `move_completed_path`          | `str`       | download_location | Destination path for move-on-completion.                                 |
| `name`                         | `str`       | (torrent name)    | Display name override.                                                   |
| `prioritize_first_last_pieces` | `bool`      | `False`           | Prioritize first and last pieces.                                        |
| `sequential_download`          | `bool`      | `False`           | Download pieces sequentially.                                            |
| `download_location`            | `str`       | (config default)  | Save path.                                                               |
| `add_paused`                   | `bool`      | `False`           | Add the torrent in paused state.                                         |
| `auto_managed`                 | `bool`      | `True`            | Whether auto-managed by the queue.                                       |
| `owner`                        | `str`       | (session user)    | Torrent owner username.                                                  |
| `shared`                       | `bool`      | `False`           | Whether shared across users.                                             |
| `stop_at_ratio`                | `bool`      | `False`           | Stop at stop ratio.                                                      |
| `stop_ratio`                   | `float`     | `2.0`             | The stop ratio.                                                          |
| `remove_at_ratio`              | `bool`      | `False`           | Remove when stop ratio is reached.                                       |
| `super_seeding`                | `bool`      | `False`           | Enable super seeding.                                                    |

---

## Filter dict

Passed as the first arg to `core.get_torrents_status`. `{}` = no filter (return all). All filter values are **lists**
even for a single value.

| Filter key      | Value type  | Behavior                                                                                                                                             |
|-----------------|-------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| `id`            | `List[str]` | Direct torrent_id match (optimized path).                                                                                                            |
| `state`         | `List[str]` | State values. `'Active'` is special: filters by `download_payload_rate > 0 or upload_payload_rate > 0`. Other values match `torrent.state` directly. |
| `keyword`       | `List[str]` | Searches name, state, tracker URL, info_hash, tracker_status, and file paths (case-insensitive, comma-separated).                                    |
| `name`          | `List[str]` | Substring match on torrent name. `::match` suffix = case-sensitive.                                                                                  |
| `tracker_host`  | `List[str]` | Matches `tracker_host` status field. `'Error'` is special: matches torrents with `'Error:'` in `tracker_status`.                                     |
| *any other key* | `List[str]` | Falls through to status field matching: `status[field] in values`.                                                                                   |

---

## Filter tree fields

Returned by `core.get_filter_tree(show_zero_hits, hide_cat)`.

**Return shape**: `{field: [(value: str, count: int), ...]}`

| Field          | Values produced                                                                                                                                                                  |
|----------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `state`        | `('All', N)`, `('Active', N)`, `('Allocating', N)`, `('Checking', N)`, `('Downloading', N)`, `('Seeding', N)`, `('Paused', N)`, `('Error', N)`, `('Queued', N)`, `('Moving', N)` |
| `tracker_host` | `('Error', N)`, `('All', total_count)`, plus one entry per unique tracker_host                                                                                                   |
| `owner`        | `('', N)` for unowned, plus one entry per unique owner                                                                                                                           |

Plugins can register additional tree fields.

**Parameters**:

- `show_zero_hits=True`: when `False`, removes entries with `count==0`.
- `hide_cat=None`: list of field names to exclude from the result.

---

## `diff=True` behavior

When `diff=True`, the response only contains keys whose values changed since the last `get_status` call for the same RPC
session. The first call with `diff=True` returns the full status dict (no previous state to diff against). Subsequent
calls return only changed keys.

## Plugin status keys

Keys not in the standard status key set are treated as plugin keys. If `keys=[]` (all keys), plugin status is also
fetched and merged into the per-torrent status dict. Plugin keys are only available when the corresponding plugin is
enabled.
