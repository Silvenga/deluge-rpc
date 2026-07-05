# deluge-rpc

This repo contains several crates (workspace):

- `deluge-cli`: A wrapper around the `deluge-rpc` crate. The cli is used for both recording for tests and for human
  consumption (automation and general CLI usage).
- `deluge-rpc`: An implementation of the deluged rpc protocol with all standard methods supported by Deluge v2.
- `deluge-rpc-mock`: A mock implementation of a deluged rpc server. Used for recordder/cassette style integration tests.

## Conventions

- All tests should execute in under 2 seconds. Longer durations suggest a bug. Notify the User if this occurs.
