# deluge-rpc-client tests

This directory (`deluge-rpc-client/tests`) contains the e2e tests for the `deluge-rpc-client` crate, testing against a mock server
(deluge-rpc-mock) that implements the deluged RPC wire protocol. The e2e tests against captures from a real deluged
server (called cassettes) in `deluge-rpc-client/fixtures`.

- Always load the `create-fixtures` skill before adding or updating these e2e tests.
- When an e2e test is added/updated, keep `deluge-rpc-client/docs/FIXTURE-COVERAGE.md` up to date.
- Name the e2e tests file after the test fixture file name. So `live_daemon.rs` for tests against the `live-daemon.json`
  fixture (the cassette).
- Cassettes are generated with the `deluge-cli` in recorder mode.
