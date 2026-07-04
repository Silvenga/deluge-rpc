# deluge-rpc

Rust client for the Deluge daemon RPC protocol. Speaks the native wire format (TLS + framed zlib + rencode) rather than
the Web UI JSON-RPC API.

## Transport

The daemon listens on TCP **58846** and requires TLS. Self-signed certificates are expected - verification is always
skipped.

Each frame has a 5-byte header followed by a zlib-compressed body:

```
+----+------------------+-----------------------------+
| v  | body_len : u32   | zlib(rencode payload)       |
| 1B | big-endian (4B)  | body_len bytes              |
+----+------------------+-----------------------------+
```

- `v` - protocol version, always `1` for Deluge 2.x.
- `body_len` - big-endian, the size of the compressed body (not the rencode payload). Frames above 16 MiB are rejected.

## Rencode

Rencode is a compact serialization format similar to `bencode`. This crate implements the subset needed by Deluge RPC:
`None`, bool, int, str, bytes, list, dict, float.

### Type codes

| Type             | Encoding                          |
|------------------|-----------------------------------|
| `None`           | `0x45`                            |
| `True`           | `0x43`                            |
| `False`          | `0x44`                            |
| int `0..=43`     | single byte `0x00..=0x2B`         |
| int `-1..=-32`   | single byte `0x46..=0x65`         |
| int8             | `0x3E` + 1 byte big-endian        |
| int16            | `0x3F` + 2 bytes big-endian       |
| int32            | `0x40` + 4 bytes big-endian       |
| int64            | `0x41` + 8 bytes big-endian       |
| float32          | `0x42` + 4 bytes big-endian       |
| str < 64B        | `0x80 + len` + bytes              |
| str >= 64B       | ASCII digits + `:` + bytes        |
| list < 64 items  | `0xC0 + len` + items              |
| list >= 64 items | `0x3B` + items + `0x7F`           |
| dict < 25 items  | `0x66 + len` + key-value pairs    |
| dict >= 25 items | `0x3C` + key-value pairs + `0x7F` |

Dicts are emitted in `BTreeMap` iteration order (deterministic). Floats encode as f32 (Deluge default); decode accepts
both f32 and f64.

## RPC envelope

### Request

```
[[request_id, method, [args...], {kwargs...}]]
```

A one-element list wrapping a 4-tuple: id (int), method name (str), positional args (list), keyword args (dict).

### Response

Three message types, discriminated by the first int:

| Type           | Tag | Shape                               |
|----------------|-----|-------------------------------------|
| `RPC_RESPONSE` | `1` | `[1, request_id, [return_value]]`   |
| `RPC_ERROR`    | `2` | `[2, exc_type, exc_msg, traceback]` |
| `RPC_EVENT`    | `3` | `[3, event_name, [args...]]`        |

Events are logged and skipped. Responses are matched by `request_id`. Return values are wrapped in a one-element list.

## Methods

| Method                     | Args                                            | Returns                 |
|----------------------------|-------------------------------------------------|-------------------------|
| `daemon.login`             | `[username, password]`, kwargs `client_version` | auth level (int)        |
| `core.get_free_space`      | `[None]`                                        | bytes (int)             |
| `core.get_torrents_status` | `[filter_dict, [field_names]]`                  | dict keyed by info hash |
| `core.remove_torrent`      | `[info_hash, remove_data]`                      | success (bool)          |

`core.get_torrents_status` takes `filter_dict` **first**, keys **second** - reversed from `web.update_ui`.
