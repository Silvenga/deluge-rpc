# Deluge v2 TCP RPC Protocol Specification

> `deluge-torrent/deluge` @ commit `e58075416dedd53636e89b1cd240f86f2e7c2ee0`

**Companion documents**:

- [SPEC-core-torrents.md](SPEC-core-torrents.md) - `core.*` torrent methods + torrent status key schema
- [SPEC-core-session.md](SPEC-core-session.md) - `core.*` session/config/network methods + config/session schemas
- [SPEC-daemon.md](SPEC-daemon.md) - `daemon.*` methods
- [SPEC-plugins.md](SPEC-plugins.md) - plugin methods + plugin config schemas

---

## 1. Transport

SSL/TLS-wrapped TCP. Five-byte header + zlib-compressed rencode body.

```
|.version (1B)|.size (4B, big-endian uint32)|.....body.....|
```

- `version` = `0x01` always
- `size` = byte length of the compressed body
- `body` = `zlib.compress(rencode.dumps(data))`
- decode: `rencode.loads(zlib.decompress(data), decode_utf8=True)`

Rencode, not JSON/Bencode.

## 2. Message envelope

### Client request (outer-wrapped)

Requests are a 1-element outer list wrapping a list of 4-tuples:

```
[[(request_id: int, method: str, args: list, kwargs: dict), ...]]
```

Multiple calls per envelope are supported (batched). Each call validated to length 4. `args` is an empty list `[]` when
no positional arguments are passed; `kwargs` defaults to `{}`. Do NOT send `[None]` as args — `@export` methods with
no positional parameters will reject it with a `WrappedException` (TypeError).

### Server response (bare tuple)

Responses are bare tuples. Only requests use the outer 1-element list wrapping.

The inner tuple is discriminated by a type tag at position 0:

| Tag | Constant       | Direction      | Inner shape                                                                                  |
|-----|----------------|----------------|----------------------------------------------------------------------------------------------|
| `1` | `RPC_RESPONSE` | server->client | `(1, request_id: int, result: any)`                                                          |
| `2` | `RPC_ERROR`    | server->client | `(2, request_id: int, exc_type_name: str, exc_args: list, exc_kwargs: dict, traceback: str)` |
| `3` | `RPC_EVENT`    | server->client | `(3, event_name: str, event_args: list)`                                                     |

> **Note**: Return values in `RPC_RESPONSE` are bare — `daemon.login` returns `10`, not `[10]`.

## 3. Auth levels

| Name                          | Int |
|-------------------------------|-----|
| `AUTH_LEVEL_NONE`             | 0   |
| `AUTH_LEVEL_READONLY`         | 1   |
| `AUTH_LEVEL_NORMAL` (default) | 5   |
| `AUTH_LEVEL_ADMIN`            | 10  |

`@export` without args -> `AUTH_LEVEL_DEFAULT` = `NORMAL` (5). Per-method overrides annotated in the method tables as
`[auth: X]`.

### Auth level mappings

Forward: `{'NONE': 0, 'READONLY': 1, 'DEFAULT': 5, 'NORMAL': 5, 'ADMIN': 10}`
Reverse: `{0: 'NONE', 1: 'READONLY', 5: 'NORMAL', 10: 'ADMIN'}`. Note: `5` maps to `'NORMAL'`, not `'DEFAULT'` (last
write wins in the source dict).

## 4. Connection lifecycle

1. TLS connect to `host:58846`.
2. `daemon.info()` -> `str` (version; pre-auth).
3. `daemon.login(username, password, client_version=...)` -> `int` (auth level; raises `BadLoginError` on failure,
   connection closed on auth level 0).
4. `daemon.set_event_interest(event_names: List[str])` -> `bool` (subscribe to events).
5. Call any method; receive `(3, "EventName", [args...])` asynchronously for subscribed events.

---

## 5. Errors

### Wire shape

```
(2, request_id: int, exc_type_name: str, exc_args: list, exc_kwargs: dict, traceback: str)
```

The inner list has **exactly 6 elements** at positions 0–5. A client parses it blindly by position:

| Position | Type   | Content                                                                                                                                                                                     |
|----------|--------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 0        | `int`  | Always `2` (RPC_ERROR tag)                                                                                                                                                                  |
| 1        | `int`  | `request_id` - matches the request that triggered the error                                                                                                                                 |
| 2        | `str`  | `exc_type_name` - Python exception class name; one of the values in the table below                                                                                                         |
| 3        | `list` | `exc_args` - positional args passed to the exception constructor. For most exceptions this is `[message_string]`. For `NotAuthorizedError` it carries `current_level` and `required_level`. |
| 4        | `dict` | `exc_kwargs` - keyword args passed to the exception constructor. Usually empty `{}`.                                                                                                        |
| 5        | `str`  | `traceback` - Python traceback string for debugging                                                                                                                                         |

### Parsing strategy

1. Read `exc_type_name` (position 2) as the primary discriminant.
2. Read `exc_args` (position 3) as a list. For single-message exceptions, `exc_args[0]` is the human-readable message.
   If `exc_args` is empty, fall back to `exc_kwargs` or the `traceback`.
3. Do not assume the list has exactly 4 elements - that was a common misparse. The wire shape is 6 elements.
4. Model `NotAuthorizedError` and `BadLoginError` as typed variants. Treat everything else as a generic "server error
   with type name + message + traceback".
5. `WrappedException` wraps any non-`DelugeError` exception raised inside an `@export` method. The original Python
   exception class name is in `exc_args`/attributes, not in `exc_type_name`.

### Error types

| `exc_type_name`          | When raised                                                                      | `exc_args` content                                       |
|--------------------------|----------------------------------------------------------------------------------|----------------------------------------------------------|
| `DelugeError`            | Base class; rarely raised directly                                               | `[message]`                                              |
| `DaemonRunningError`     | Second daemon start attempt                                                      | `[message]`                                              |
| `InvalidTorrentError`    | Bad torrent file/data; also raised by `remove_torrent` if torrent_id not found   | `[message]`                                              |
| `AddTorrentError`        | `add_torrent_files` partial failure (returned in result list, not always raised) | `[message]`                                              |
| `InvalidPathError`       | Bad filesystem path                                                              | `[message]`                                              |
| `WrappedException`       | Catch-all: any non-`DelugeError` raised inside an `@export` method               | `[message]`; original exception class name in attributes |
| `NotAuthorizedError`     | Auth level too low for the called method                                         | Carries `current_level` and `required_level`             |
| `BadLoginError`          | Wrong username/password at `daemon.login`                                        | `[message]`                                              |
| `AuthenticationRequired` | Calling a method before `daemon.login`                                           | `[message]`                                              |
| `AuthManagerError`       | Auth manager internal failure                                                    | `[message]`                                              |
| `InvalidHashError`       | Invalid torrent hash passed to a method                                          | `[message]`; `.method` attribute                         |
| `IncompatibleClient`     | Client/daemon version mismatch                                                   | `[message]`; `.daemon_version` attribute                 |

---

## 6. Events

Delivered as `RPC_EVENT` (tag `3`) messages:

```
(3, event_name: str, event_args: list)
```

`event_name` equals the event class name. Subscribe via `daemon.set_event_interest(["EventName", ...])`. Plugin events
only fire when the owning plugin is enabled.

### Core events - 22

| Event name                   | Args                        | Description                                                                                         |
|------------------------------|-----------------------------|-----------------------------------------------------------------------------------------------------|
| `TorrentAddedEvent`          | `[torrent_id, from_state]`  | A torrent was added to the session. `from_state` indicates whether it was added from a saved state. |
| `TorrentRemovedEvent`        | `[torrent_id]`              | A torrent was removed from the session.                                                             |
| `PreTorrentRemovedEvent`     | `[torrent_id]`              | Emitted before a torrent is removed - last chance to access its data.                               |
| `TorrentStateChangedEvent`   | `[torrent_id, state]`       | A torrent's state changed (e.g. Downloading -> Paused).                                             |
| `TorrentTrackerStatusEvent`  | `[torrent_id, status]`      | A torrent's tracker status changed (e.g. "Announce OK").                                            |
| `TorrentQueueChangedEvent`   | `[]`                        | The queue order changed.                                                                            |
| `TorrentFolderRenamedEvent`  | `[torrent_id, old, new]`    | A folder within a torrent was renamed.                                                              |
| `TorrentFileRenamedEvent`    | `[torrent_id, index, name]` | A file within a torrent was renamed.                                                                |
| `TorrentFinishedEvent`       | `[torrent_id]`              | A torrent finished downloading.                                                                     |
| `TorrentResumedEvent`        | `[torrent_id]`              | A torrent was resumed from pause.                                                                   |
| `TorrentFileCompletedEvent`  | `[torrent_id, index]`       | A single file within a torrent completed.                                                           |
| `TorrentStorageMovedEvent`   | `[torrent_id, path]`        | A torrent's storage location was moved.                                                             |
| `CreateTorrentProgressEvent` | `[piece_count, num_pieces]` | Progress update while creating a torrent.                                                           |
| `NewVersionAvailableEvent`   | `[new_release]`             | A new Deluge release is available.                                                                  |
| `SessionStartedEvent`        | `[]`                        | The session started.                                                                                |
| `SessionPausedEvent`         | `[]`                        | The session was paused.                                                                             |
| `SessionResumedEvent`        | `[]`                        | The session was resumed.                                                                            |
| `ConfigValueChangedEvent`    | `[key, value]`              | A config value changed.                                                                             |
| `PluginEnabledEvent`         | `[plugin_name]`             | A plugin was enabled.                                                                               |
| `PluginDisabledEvent`        | `[plugin_name]`             | A plugin was disabled.                                                                              |
| `ClientDisconnectedEvent`    | `[session_id]`              | A client disconnected from the daemon.                                                              |
| `ExternalIPEvent`            | `[external_ip]`             | The external IP address was determined.                                                             |

### Plugin events - 4

| Event name                   | Args                           | Description                                                                              |
|------------------------------|--------------------------------|------------------------------------------------------------------------------------------|
| `AutoaddOptionsChangedEvent` | `[]`                           | AutoAdd plugin options changed.                                                          |
| `ExecuteCommandAddedEvent`   | `[command_id, event, command]` | An Execute plugin command was added. `event` is `"complete"`, `"added"`, or `"removed"`. |
| `ExecuteCommandRemovedEvent` | `[command_id]`                 | An Execute plugin command was removed.                                                   |
| `SchedulerEvent`             | `[colour]`                     | Scheduler plugin state changed. `colour` is `"Green"`, `"Yellow"`, or `"Red"`.           |

---

## 7. Method index

112 methods total.

| Prefix                           | Count   | Document                                       |
|----------------------------------|---------|------------------------------------------------|
| `core.` (torrents)               | 30      | [SPEC-core-torrents.md](SPEC-core-torrents.md) |
| `core.` (session/config/network) | 30      | [SPEC-core-session.md](SPEC-core-session.md)   |
| `daemon.`                        | 7       | [SPEC-daemon.md](SPEC-daemon.md)               |
| `autoadd.`                       | 10      | [SPEC-plugins.md](SPEC-plugins.md)             |
| `blocklist.`                     | 4       | [SPEC-plugins.md](SPEC-plugins.md)             |
| `execute.`                       | 4       | [SPEC-plugins.md](SPEC-plugins.md)             |
| `extractor.`                     | 2       | [SPEC-plugins.md](SPEC-plugins.md)             |
| `label.`                         | 8       | [SPEC-plugins.md](SPEC-plugins.md)             |
| `notifications.`                 | 3       | [SPEC-plugins.md](SPEC-plugins.md)             |
| `scheduler.`                     | 3       | [SPEC-plugins.md](SPEC-plugins.md)             |
| `stats.`                         | 6       | [SPEC-plugins.md](SPEC-plugins.md)             |
| `toggle.`                        | 2       | [SPEC-plugins.md](SPEC-plugins.md)             |
| `webui.`                         | 3       | [SPEC-plugins.md](SPEC-plugins.md)             |
| **Total**                        | **112** |                                                |
