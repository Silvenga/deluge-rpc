//! E2e tests for torrent lifecycle RPC methods against a cassette replay server.

use deluge_rpc_client::{CoreTorrentRpc, DelugeClientBuilder};
use deluge_rpc_mock::{Cassette, Matcher, ReplayServer};
use std::fs;
use std::path::PathBuf;

const FIXTURE: &str = "torrent-lifecycle.json";
const TORRENT_ID: &str = "83d40fa6191f96716d36bee7bc04274ee792ec45";

fn load_fixture() -> Cassette {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(FIXTURE);
    let json = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {FIXTURE}: {e}"));
    Cassette::from_json_str(&json)
        .unwrap_or_else(|e| panic!("failed to parse fixture {FIXTURE}: {e}"))
}

async fn start_replay(cassette: Cassette) -> ReplayServer {
    ReplayServer::start(Matcher::new(cassette.interactions))
        .await
        .expect("start replay server")
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_lifecycle_cassette_then_get_torrent_status_returns_name() {
    let server = start_replay(load_fixture()).await;

    let client = DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build();

    let status = client
        .core()
        .torrents
        .get_torrent_status(
            TORRENT_ID,
            &["name".into(), "state".into(), "progress".into()],
            false,
        )
        .await
        .expect("core.get_torrent_status");

    assert_eq!(status.name, "debian-12.0.0-amd64-netinst.iso");
    assert_eq!(status.state, "Downloading");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_lifecycle_cassette_then_remove_torrent_returns_true() {
    let server = start_replay(load_fixture()).await;

    let client = DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build();

    let result = client
        .core()
        .torrents
        .remove_torrent(TORRENT_ID, true)
        .await
        .expect("core.remove_torrent");

    assert!(result, "remove_torrent should return true");
}
