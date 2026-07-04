# Deluge v2 TCP RPC - Client Implementation Recommendations

Issues found while researching the protocol that affect a Rust client. Ordered by severity.

---

## 1. Error envelope shape mismatch (HIGH)

**Issue**: The current `deluge-rpc/src/wire.rs:rpc_error()` decodes positions 1, 2, 3 of the inner list as
`(exc_type, exc_msg, traceback)` - a 4-element read. The actual server wire shape is **6 elements**:

```
(2, request_id, exc_type_name, exc_args, exc_kwargs, traceback)
```

`exc_args` is a **list/tuple** and `exc_kwargs` is a **dict**, not strings. Current code happens to not crash only
because `field_as_str` falls through to `format!("{other:?}")` for non-string values, producing garbage like
`["bad password"]` instead of `bad password`.

**Fix**: decode the error by position per the spec in [SPEC.md](SPEC.md#5-errors). Position 2 is the type name (str),
position 3 is `exc_args` (list - `exc_args[0]` is the message for most errors), position 4 is `exc_kwargs` (dict),
position 5 is the traceback (str).

**Impact**: every error response is currently mis-parsed. Silent today because errors are rare and the fallback string
still surfaces something, but structured error handling (typed `BadLoginError` vs `NotAuthorizedError`) is impossible
without the fix.

## 2. Plugin method availability is runtime-variable (MEDIUM)

**Issue**: Plugin methods (`label.*`, `autoadd.*`, `execute.*`, etc.) only exist when the corresponding plugin is
enabled on the daemon. Calling a method whose plugin is disabled raises `WrappedException` (wrapping a Python
`AttributeError`).

**Recommendation**:

- Before calling any plugin method, check `core.get_enabled_plugins()` and gate the call.
- Alternatively, call `daemon.get_method_list()` once at startup to get the live set of registered method names and fail
  fast on unsupported calls.
- Treat plugin method absence as a recoverable condition, not a transport error.

## 3. `daemon.info` / `daemon.login` / `daemon.set_event_interest` are not in `get_method_list()` (MEDIUM)

**Issue**: These three are hardcoded in `RPCServer.dispatch()` and bypass the `@export` registration. They are therefore
absent from `daemon.get_method_list()`'s output, despite being callable.

**Recommendation**: hardcode these three as always-available in the client's method table regardless of
`get_method_list()` results. Do not use `get_method_list()` to validate them.

## 4. No protocol version negotiation (LOW)

**Issue**: Method names are bare strings with no namespace. Deluge v1 vs v2 differ in available methods (v2 added
`add_torrent_files`, `prefetch_magnet_metadata`, etc.) but the protocol exposes no formal version negotiation - only
`daemon.info` (returns version string) and `daemon.get_version`.

**Recommendation**: at connection time, call `daemon.info()` (pre-auth) or `daemon.get_version()` (post-auth) and parse
the version string. Refuse or warn if the daemon is not v2.x. Method availability can then be cross-checked against
`daemon.get_method_list()`.

## 5. `core.get_torrents_status` arg order (LOW - already handled)

**Note**: `core.get_torrents_status(filter_dict, keys)` - `filter_dict` is FIRST, `keys` is SECOND. This is **reversed**
from the web UI's `web.update_ui` argument order. The local `client.rs` already documents and implements this correctly.
Do not "fix" it to match the web UI order.

## 6. Batched calls supported but unused (LOW)

**Note**: The request envelope wraps a **list** of 4-tuples, allowing multiple RPC calls per send. The current client
sends one call per request. If latency becomes an issue (e.g. fetching per-torrent status for many torrents), batching
could help. Not actionable now.

## 7. Auth level awareness (LOW)

**Note**: `core.get_known_accounts`, `create_account`, `update_account`, `remove_account` require `AUTH_LEVEL_ADMIN` (
10). `daemon.authorized_call` requires `READONLY` (1) but is itself the mechanism to check authorization for any method.
If the client connects with a non-admin user, expect `NotAuthorizedError` from admin-gated calls. The default
`localclient` user on a standard install has ADMIN, but do not assume this in general.

## 8. Deferred / async returns (LOW)

**Note**: Many `core.*` methods return Twisted `Deferred` objects server-side. Over the wire these resolve to the
deferred's eventual result (the daemon awaits resolution before sending the response). From the Rust client's
perspective these look like synchronous request/response - no special handling needed, but some methods (e.g.
`core.test_listen_port`, `core.enable_plugin`) may take noticeably longer to respond because they wait on async
server-side work.
