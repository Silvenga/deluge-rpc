# SPEC - `daemon.*` Methods

> `deluge-torrent/deluge` @ commit `e58075416dedd53636e89b1cd240f86f2e7c2ee0`

## Exported methods

| Method                   | Args                | Returns       | Auth         | Description                                                                                                                                                                                                                                                 |
|--------------------------|---------------------|---------------|--------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `daemon.shutdown`        | `(*args, **kwargs)` | `DelayedCall` | NORMAL       | Shuts down the daemon. The reactor stops immediately after scheduling the stop - the client may never receive the response. Do not block on the response; close the connection after sending the request.                                                   |
| `daemon.get_method_list` | `()`                | `List[str]`   | NORMAL       | Returns all registered RPC method names. Each string is `"<object_name>.<method_name>"` (e.g. `"core.add_torrent_file"`, `"daemon.get_version"`, `"autoadd.add"`). Only includes `@export`-registered methods - excludes the three hardcoded methods below. |
| `daemon.get_version`     | `()`                | `str`         | NORMAL       | Returns the daemon version string (e.g. `"2.1.1"`) via `deluge.common.get_version()`.                                                                                                                                                                       |
| `daemon.authorized_call` | `(rpc: str)`        | `bool`        | **READONLY** | Checks whether the current session's auth level is sufficient to call the named RPC method. Returns `False` if the method doesn't exist or if the session auth level is below the method's required level. Use to pre-check authorization before calling.   |

---

## Hardcoded methods (in `RPCServer.dispatch()`)

These are **not** `@export`-decorated and **not** listed by `daemon.get_method_list()`. They are always available.

| Method                      | Args                                                     | Returns                           | Auth       | Description                                                                                                                                                                                                                                                                |
|-----------------------------|----------------------------------------------------------|-----------------------------------|------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `daemon.info`               | `()`                                                     | `str` (version)                   | pre-auth   | Returns the daemon version string. Used in the initial handshake before login. Same value as `daemon.get_version`.                                                                                                                                                         |
| `daemon.login`              | `(username: str, password: str, *, client_version: str)` | `int` (auth level)                | pre-auth   | Authenticates the session. `client_version` is passed as a kwarg. Returns `0`/`1`/`5`/`10` (auth level). Raises `BadLoginError` on bad credentials - the exception is sent as `RPC_ERROR` and the connection is closed. A return of `0` (NONE) also closes the connection. |
| `daemon.set_event_interest` | `(event_names: List[str])`                               | `bool` (always `True` on success) | post-login | Subscribes the current session to the listed event names. This is a full-replace operation - each call sets the session's interest list. To add events, re-send the full desired list. There is no incremental add/remove.                                                 |

---

## Auth level values

Returned by `daemon.login` and used by `daemon.authorized_call`:

| Int  | Name                 | Notes                        |
|------|----------------------|------------------------------|
| `0`  | `NONE`               | Connection closed by server. |
| `1`  | `READONLY`           | Read-only access.            |
| `5`  | `NORMAL` / `DEFAULT` | Standard access.             |
| `10` | `ADMIN`              | Full access.                 |

---

## Implementation notes

- **Hardcoded methods aren't in `get_method_list()`**: Do not use `get_method_list()` to validate `daemon.info`,
  `daemon.login`, or `daemon.set_event_interest`. Treat them as always-available.
- **`daemon.shutdown` response is unreliable**: The reactor stops before the response is flushed. Don't block on the
  response; close the connection after sending the request.
- **`daemon.login` failure modes**: Bad credentials -> `BadLoginError` (RPC_ERROR). Auth level 0 -> connection closed by
  server (no response). Successful login -> `int` auth level.
- **`daemon.set_event_interest` is full-replace**: Each call sets the session's interest list. Track the current list
  client-side and re-send with additions.
- **`daemon.authorized_call` checks against `get_method_list()`**: Since the hardcoded methods aren't in that list,
  `authorized_call("daemon.info")` returns `False` even though the method is callable. Only use `authorized_call` for
  `@export`-registered methods.
