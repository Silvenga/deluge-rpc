---
name: create-fixtures
description: >
  Create cassette fixtures and e2e tests for the deluge-rpc crate.
  Use when recording real Deluge daemon responses into replayable cassettes or writing integration tests that replay them.
---

# Create Fixtures

Record real Deluge daemon responses into cassette fixtures and write e2e tests that replay them.

## Prerequisites

- `deluge-cli` built: `cargo build -p deluge-cli`
- Access to a live Deluge daemon (host, port, user, password). The User will provide these connection settings.

## Step 0: Ensure CLI Sub-Command Exists

Before recording, confirm the `deluge-cli` subcommand for the target method exists. Typed subcommands live in
`deluge-cli/src/commands/{daemon,core,plugins}.rs`. If no typed subcommand covers the method, use the generic
`call <method> <args-json> <kwargs-json>` subcommand instead (see Step 1).

## Step 1: Record a cassette

Use `deluge-cli --record` to capture RPC responses. The recorder appends to an existing cassette, so run multiple
commands to accumulate interactions into one file:

```bash
FX=<fixture-name> # e.g. "session-status"
CASSETTE="deluge-rpc-client/fixtures/${FX}.json"

# Delete any existing fixture to start fresh
rm -f "$CASSETTE"

# Record each method you want in the cassette
# Typed subcommands:
./target/debug/deluge-cli --host <host> --port <port> --user <user> --pass <pass> \
  --record "$CASSETTE" daemon info

./target/debug/deluge-cli --host <host> --port <port> --user <user> --pass <pass> \
  --record "$CASSETTE" core free-space

./target/debug/deluge-cli --host <host> --port <port> --user <user> --pass <pass> \
  --record "$CASSETTE" core torrents list

# Generic call command (for methods without a typed subcommand):
./target/debug/deluge-cli --host <host> --port <port> --user <user> --pass <pass> \
  --record "$CASSETTE" call core.get_session_status '[]' '{}'

# Args are plain JSON: '["arg1", 42]' not tagged RencodeValue.
```

Verify the cassette:

```bash
python3 -c "import json; d=json.load(open('$CASSETTE')); print(len(d['interactions']), 'interactions'); [print(' ', i['request']['method']) for i in d['interactions']]"
```

The cassette must contain zero `daemon.login` interactions. The mock server auto-serves login.

## Step 2: Write the e2e test

Create `deluge-rpc-client/tests/<fixture_name>.rs` (named after the fixture file, e.g. `live_daemon.rs` for
`live-daemon.json`). Copy this pattern:

```rust
use deluge_rpc_client::{DaemonRpc, DelugeClientBuilder};  // import traits you call
use deluge_rpc_mock::{Cassette, Matcher, ReplayServer};
use std::fs;
use std::path::PathBuf;

fn load_fixture() -> Cassette {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("<fixture-name>.json");
    let json = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture: {e}"));
    Cassette::from_json_str(&json)
        .unwrap_or_else(|e| panic!("failed to parse fixture: {e}"))
}

async fn start_replay(cassette: Cassette) -> ReplayServer {
    ReplayServer::start(Matcher::new(cassette.interactions))
        .await
        .expect("start replay server")
}

#[tokio::test(flavor = "multi_thread")]
async fn when_<condition>_then_<expected>() {
    let server = start_replay(load_fixture()).await;

    let client = DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build();

    // Call typed methods and assert on the response
    let info = client.daemon().info().await.expect("daemon.info");
    assert_eq!(info, "2.1.2.dev0");
}
```

Key points:

- Import the specific traits you call (e.g. `DaemonRpc`, `CoreTorrentRpc`, `CoreSessionRpc`)
- Use `"any"` for username/password — the mock auto-serves login regardless
- Test names follow `when_<condition>_then_<expected>` convention
- One test per logical assertion; group related assertions if they share a cassette

## Step 3: Verify

```bash
cargo test -p deluge-rpc-client --test <fixture_name>
cargo clippy -p deluge-rpc-client --test <fixture_name> -- -D warnings
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Step 4: Update FIXTURE-COVERAGE.md

In `deluge-rpc-client/docs/FIXTURE-COVERAGE.md`, update the Cassette column for each method now covered by the
new fixture, and update the "Cassette e2e" counts in the Summary table at the bottom.

## Cassette format reference

Cassettes are JSON with tagged RencodeValue encoding for args/kwargs/response values:

- `{"type":"str","value":"hello"}` — string
- `{"type":"int","value":42}` — integer
- `{"type":"bool","value":true}` — boolean
- `{"type":"none"}` — null
- `{"type":"list","value":[...]}` — array (elements are tagged recursively)
- `{"type":"dict","value":[{"key":...,"value":...},...]}` — object (array of pairs, preserves key order)

Do not hand-edit cassettes. Record them from a real daemon using `deluge-cli --record`.

## Constraints

- NEVER commit any credentials the User provides. ALWAYS verify that the cassette fixtures do not contain the password.
