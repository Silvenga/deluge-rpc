//! Shared test helpers for cassette-replay e2e tests.

use deluge_rpc_client::DelugeClientBuilder;
use deluge_rpc_mock::{Cassette, Matcher, ReplayServer};
use std::fs;
use std::path::PathBuf;

pub fn load_fixture(name: &str) -> Cassette {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name);
    let json =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"));
    Cassette::from_json_str(&json).unwrap_or_else(|e| panic!("failed to parse fixture {name}: {e}"))
}

pub async fn start_replay(cassette: Cassette) -> ReplayServer {
    ReplayServer::start(Matcher::new(cassette.interactions))
        .await
        .expect("start replay server")
}

pub async fn build_client(server: &ReplayServer) -> deluge_rpc_client::DelugeClient {
    DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build()
}
