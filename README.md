# deluge-rpc

A fully-featured Rust library for the Deluge RPC protocol. Speaks the native wire format (TLS + framed zlib + rencode)
not the JSON-RPC API used by the Deluge Web UI. Also `deluge-rpc-rencode`, a serde compatible rencode implementation
(Deluge's `bencode`-like binary message format).

This project also packages a CLI tool (`deluge-cli`) for interacting with Deluge instances - for scripting or general
testing `deluge-rpc-client`.
