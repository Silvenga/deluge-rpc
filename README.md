# deluge-rpc

A fully-featured Rust library for the Deluge RPC protocol. Speaks the native wire format (TLS + framed zlib + rencode)
not the JSON-RPC API used by the Deluge Web UI. This project also provides `deluge-rpc-rencode`, a serde compatible
rencode implementation (Deluge's `bencode`-like binary message format).

A CLI tool (`deluge-cli`) is also available for interacting with Deluge instances - for scripting or general testing
`deluge-rpc-client`.

## CLI

The CLI is a thin wrapper across the `deluge-rpc-client` library, with some quality of life improves.

```text
CLI client for the Deluge daemon RPC protocol

Usage: deluge-cli [OPTIONS] --password <PASSWORD> <COMMAND>

Commands:
  call    Call a raw RPC method directly by name with JSON args/kwargs
  daemon  `daemon.*` methods (e.g., daemon info, version, methods, shutdown)
  core    `core.*` methods (e.g., torrents, session, config, plugins)
  plugin  Plugin RPC methods (e.g., label, etc.). Plugin methods require the plugin to be enabled
  status  High-level status overview
  help    Print this message or the help of the given subcommand(s)

Options:
  -H, --host <HOST>          The deluge daemon host [env: DELUGE_HOST=] [default: 127.0.0.1]
  -P, --port <PORT>          The deluge daemon port [env: DELUGE_PORT=] [default: 58846]
  -u, --username <USERNAME>  The deluge daemon username [env: DELUGE_USERNAME=] [default: localclient]
  -p, --password <PASSWORD>  The deluge daemon password [env: DELUGE_PASSWORD=]
      --record <RECORD>      File path to record RPC calls to
  -h, --help                 Print help
  -V, --version              Print version
```

Either download from [releases](https://github.com/Silvenga/deluge-rpc/releases) or build from source.

```bash
cargo install --git https://github.com/Silvenga/deluge-rpc.git --locked
```

Don't consider the output to be stable until v1.0. Likely mostly useful to script modifications to deluged.

## Library

The docs are going to be your friend, there are 112 RPC methods that aren't deprecated.

- [crates.io/crates/deluge-rpc-client](https://crates.io/crates/deluge-rpc-client)
- [docs.rs/deluge-rpc-client](https://docs.rs/deluge-rpc-client/latest/deluge_rpc_client/)

Generally, your entrypoint will be the `DelugeClientBuilder`, producing a `DelugeClient`. See
[`deluge-rpc-client/README.md`](deluge-rpc-client/README.md) for examples.
