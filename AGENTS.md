# deluge-rpc

This repo contains several crates (workspace):

- `deluge-cli`: A wrapper around the `deluge-rpc-client` crate. The cli is used for both recording for tests and for human
  consumption (automation and general CLI usage).
- `deluge-rpc-rencode`: Houses the core type `RencodeValue` and serde support for Reencode (the data exchange format used by
  deluge).
- `deluge-rpc-client`: An implementation of the deluged rpc protocol with all standard methods supported by Deluge v2.
- `deluge-rpc-models`: Houses the typed input/output models for deluge rpc methods.
- `deluge-rpc-mock`: A mock implementation of a deluged rpc server. Used for recordder/cassette style integration tests.

## Conventions

- All tests should execute in under 2 seconds. Longer durations suggest a bug. Notify the User if this occurs.
- Test should be written without sleeps/delays to work around race-conditions.
