# deluge-rpc-client

Client library for the Deluge daemon RPC protocol (TLS + framed zlib + rencode).

Also see [`deluge-cli`](https://github.com/Silvenga/deluge-rpc), a CLI that exposes `deluge-rpc-client` - imagined for
debugging and quick scripting.

## Features

- Connects directly to the Deluge daemon over the Deluge RPC protocol, no Web UI required.
- Provides strongly typed request/response models and strongly typed fn's to call all standard RPC methods. RPC errors
  are raised as typed Rust errors.
- Concurrent RPC calls are multiplexed using the same connection.
- Await/Tokio native, a background task is spawned to prevent backpressure on the deluge daemon.

## Usage

Build `DelugeClient` using `DelugeClientBuilder`. Like, HTTP clients, it's best to share the same `DelugeClient`
instance across threads. Connecting and re-connecting are lazy. If the client disconnects mid call, that call will raise
an error. The next call will attempt to connect and login using a new connection.

All models are re-exported from [`deluge-rpc-models`](https://crates.io/crates/deluge-rpc-models).

```rust
use deluge_rpc_client::{DelugeClientBuilder, models::FilterDict};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Build the client with the daemon's host, port, and credentials.
    // The default `localclient` user is created on install (see Authentication below).
    let client = DelugeClientBuilder::new(
        "127.0.0.1",
        58846,
        "localclient",
        "password",
    )
        .build(); // Why does rustfmt do this? Is this ugly to anyone else?

    // List all torrents. An empty FilterDict returns every torrent.
    // The `CoreTorrentRpc` trait must be in scope for typed torrent methods.
    let entries = client
        .core
        .torrents
        .get_torrents_status(&FilterDict::default(), &[], false)
        .await?;

    for entry in entries {
        println!("{:<40} {:?} {:.1}%", entry.info_hash, entry.status.state, entry.status.progress);
    }

    Ok(())
}
```

> Using the default features is recommended. The `recorder` feature enables recording of RPC calls for testing purposes.

## Authentication

Authentication is required to connect via RPC. Typically, a default user `localclient` is created on installation. You
can configure authentication by modifying the `auth` file.

The `auth` file has one user per line, with the format:

```text
<user>:<password>:<auth level, 10 is admin>
```

> This `auth` file is in the root of the configuration directory, e.g., `~/.config/deluge/` on Linux.

## Custom Calls

Since plugins can add additional methods, so the strongly typed functions won't be enough. For these cases, you can use
the `.call(...)` fn to make RPC calls directly.

```rust
use deluge_rpc_client::{DelugeClientBuilder, DelugeRpcRequest, RencodeValue};
use std::collections::BTreeMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = DelugeClientBuilder::new(
        "127.0.0.1",
        58846,
        "localclient",
        "password",
    )
    .build();

    // Call a plugin-provided method not covered by the typed API.
    // Args are positional (a Vec<RencodeValue>), kwargs are keyed. Both are optional.
    let mut kwargs = BTreeMap::new();
    kwargs.insert(RencodeValue::Str("apply".into()), RencodeValue::Bool(true));

    let request = DelugeRpcRequest::new("label.set_torrent")
        .with_args(vec![
            RencodeValue::Str("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".into()),
            RencodeValue::Str("my-label".into()),
        ])
        .with_kwargs(kwargs);

    let response = client.call(request).await?;
    println!("{response:?}");

    Ok(())
}
```
