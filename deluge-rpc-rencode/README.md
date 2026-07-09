## deluge-rpc-rencode

Rencode is a compact serialization format similar to `bencode`. This crate implements the subset needed by Deluge RPC:
`None`, bool, int, str, bytes, list, dict, float.

You likely want to use [`deluge-rpc-client`](https://crates.io/crates/deluge-rpc-client) directly instead.

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

- Dicts are emitted in `BTreeMap` iteration order (deterministic).
- Floats encode as f32 (Deluge default). Decode accepts both f32 and f64.

## RPC envelope

### Request

```
[[request_id, method, [args...], {kwargs...}]]
```

A one-element list wrapping a 4-tuple: id (int), method name (str), positional args (list), keyword args (dict).

### Response

Responses are bare tuples (only requests use the outer 1-element list).

Three message types, discriminated by the first int:

| Type           | Tag | Shape                                                        |
|----------------|-----|--------------------------------------------------------------|
| `RPC_RESPONSE` | `1` | `[1, request_id, return_value]`                              |
| `RPC_ERROR`    | `2` | `[2, request_id, exc_type, exc_args, exc_kwargs, traceback]` |
| `RPC_EVENT`    | `3` | `[3, event_name, [args...]]`                                 |

Responses are matched by `request_id`. Return values are bare - `daemon.login` returns `10`, not `[10]`.
