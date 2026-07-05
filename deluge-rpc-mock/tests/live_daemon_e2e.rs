//! E2E tests replaying cassettes recorded from a live Deluge daemon.
//!
//! These tests load JSON cassettes from `fixtures/` and replay them against
//! a `DelugeClient` connected to a `ReplayServer`. The cassettes were
//! recorded from a real Deluge v2.1.2.dev0 daemon (libtorrent 2.0.10.0)
//! using `deluge-cli --record`.

use deluge_rpc::{
    CoreConfigRpc, CorePluginRpc, CoreSessionRpc, CoreTorrentRpc, DaemonRpc, DelugeClient,
};
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
async fn when_live_daemon_cassette_then_daemon_info_returns_version() {
    let cassette = load_fixture("live-daemon.json");
    let server = start_replay(cassette).await;

    let client = DelugeClient::connect(&server.host(), server.port(), "any", "any")
        .await
        .expect("connect");

    let info = client.daemon().info().await.expect("daemon.info");
    assert_eq!(info, "2.1.2.dev0");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_get_version_returns_version() {
    let cassette = load_fixture("live-daemon.json");
    let server = start_replay(cassette).await;

    let client = DelugeClient::connect(&server.host(), server.port(), "any", "any")
        .await
        .expect("connect");

    let version = client.daemon().get_version().await.expect("daemon.get_version");
    assert_eq!(version, "2.1.2.dev0");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_free_space_returns_bytes() {
    let cassette = load_fixture("live-daemon.json");
    let server = start_replay(cassette).await;

    let client = DelugeClient::connect(&server.host(), server.port(), "any", "any")
        .await
        .expect("connect");

    let space = client.core().session.get_free_space(None).await.expect("core.get_free_space");
    assert!(space > 0, "free space should be positive, got {space}");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_torrents_list_returns_entries() {
    let cassette = load_fixture("live-daemon.json");
    let server = start_replay(cassette).await;

    let client = DelugeClient::connect(&server.host(), server.port(), "any", "any")
        .await
        .expect("connect");

    let entries = client
        .core()
        .torrents
        .get_torrents_status(&Default::default(), &[], false)
        .await
        .expect("core.get_torrents_status");

    assert!(!entries.is_empty(), "should have at least one torrent");
    let entry = &entries[0];
    assert!(!entry.info_hash.is_empty(), "info_hash should not be empty");
    assert!(!entry.status.name.is_empty(), "name should not be empty");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_session_status_has_metrics() {
    let cassette = load_fixture("live-daemon.json");
    let server = start_replay(cassette).await;

    let client = DelugeClient::connect(&server.host(), server.port(), "any", "any")
        .await
        .expect("connect");

    let status = client
        .core()
        .session
        .get_session_status(&[])
        .await
        .expect("core.get_session_status");

    assert!(status.extra.contains_key("dht.dht_nodes") || !status.extra.is_empty(),
        "session status should have overflow metrics");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_config_deserializes() {
    let cassette = load_fixture("live-daemon.json");
    let server = start_replay(cassette).await;

    let client = DelugeClient::connect(&server.host(), server.port(), "any", "any")
        .await
        .expect("connect");

    let config = client
        .core()
        .config
        .get_config()
        .await
        .expect("core.get_config");

    assert!(config.daemon_port == 58846, "daemon_port should be 58846, got {}", config.daemon_port);
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_enabled_plugins_returns_list() {
    let cassette = load_fixture("live-daemon.json");
    let server = start_replay(cassette).await;

    let client = DelugeClient::connect(&server.host(), server.port(), "any", "any")
        .await
        .expect("connect");

    let plugins = client
        .core()
        .plugins
        .get_enabled_plugins()
        .await
        .expect("core.get_enabled_plugins");

    assert!(!plugins.is_empty(), "should have at least one enabled plugin");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_lifecycle_cassette_then_get_torrent_status_returns_name() {
    let cassette = load_fixture("torrent-lifecycle.json");
    let server = start_replay(cassette).await;

    let client = DelugeClient::connect(&server.host(), server.port(), "any", "any")
        .await
        .expect("connect");

    let torrent_id = "83d40fa6191f96716d36bee7bc04274ee792ec45";
    let status = client
        .core()
        .torrents
        .get_torrent_status(torrent_id, &["name".into(), "state".into(), "progress".into()], false)
        .await
        .expect("core.get_torrent_status");

    assert_eq!(status.name, "debian-12.0.0-amd64-netinst.iso");
    assert_eq!(status.state, "Downloading");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_lifecycle_cassette_then_remove_torrent_returns_true() {
    let cassette = load_fixture("torrent-lifecycle.json");
    let server = start_replay(cassette).await;

    let client = DelugeClient::connect(&server.host(), server.port(), "any", "any")
        .await
        .expect("connect");

    let torrent_id = "83d40fa6191f96716d36bee7bc04274ee792ec45";
    let result = client
        .core()
        .torrents
        .remove_torrent(torrent_id, true)
        .await
        .expect("core.remove_torrent");

    assert!(result, "remove_torrent should return true");
}
