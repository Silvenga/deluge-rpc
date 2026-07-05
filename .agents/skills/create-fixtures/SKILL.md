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

## Step 1: Record a cassette

Use `deluge-cli --record` to capture RPC responses. The recorder appends to an existing cassette, so run multiple
commands to accumulate interactions into one file:

```bash
FX=<fixture-name> # e.g. "session-status"
CASSETTE="deluge-rpc-mock/fixtures/${FX}.json"

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

Create `deluge-rpc-mock/tests/<fixture_name>_e2e.rs`. Copy this pattern:

```rust
use deluge_rpc::{DaemonRpc, DelugeClient};  // import traits you call
use deluge_rpc_mock::{Cassette, Matcher, ReplayServer};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

fn load_fixture(name: &str) -> Cassette {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name);
    let json = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"));
    Cassette::from_json_str(&json)
        .unwrap_or_else(|e| panic!("failed to parse fixture {name}: {e}"))
}

async fn start_replay(cassette: Cassette) -> ReplayServer {
    let matcher = Matcher::new(cassette.interactions);
    ReplayServer::start(Arc::new(matcher))
        .await
        .expect("start replay server")
}

#[tokio::test(flavor = "multi_thread")]
async fn when_<condition>_then_<expected>() {
    let cassette = load_fixture("<fixture-name>.json");
    let server = start_replay(cassette).await;

    let client = DelugeClient::connect( & server.host(), server.port(), "any", "any")
    .await
    .expect("connect");

    // Call typed methods and assert on the response
    let info = client.daemon().info().await.expect("daemon.info");
    assert_eq ! (info, "2.1.2.dev0");
}
```

Key points:

- Import the specific traits you call (e.g. `DaemonRpc`, `CoreTorrentRpc`, `CoreSessionRpc`)
- Use `"any"` for username/password — the mock auto-serves login regardless
- Test names follow `when_<condition>_then_<expected>` convention
- One test per logical assertion; group related assertions if they share a cassette

## Step 3: Verify

```bash
cargo test -p deluge-rpc-mock --test <fixture_name>_e2e
cargo clippy -p deluge-rpc-mock --test <fixture_name>_e2e -- -D warnings
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Step 4: Update METHOD-COVERAGE.md

In `deluge-rpc/docs/METHOD-COVERAGE.md`, add the fixture to the "Cassette fixtures" table and mark the methods as
verified in the "Verification status" section.

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
