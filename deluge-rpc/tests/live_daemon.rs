//! E2e tests against a cassette recorded from a real Deluge daemon.

use deluge_rpc::{
    CoreConfigRpc, CorePluginRpc, CoreSessionRpc, CoreTorrentRpc, DaemonRpc, DelugeClientBuilder,
};
use deluge_rpc_mock::{Cassette, Matcher, ReplayServer};
use std::fs;
use std::path::PathBuf;

const FIXTURE: &str = "live-daemon.json";

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
async fn when_live_daemon_cassette_then_daemon_info_returns_version() {
    let server = start_replay(load_fixture()).await;

    let client = DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build();

    let info = client.daemon().info().await.expect("daemon.info");
    assert_eq!(info, "2.1.2.dev0");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_get_version_returns_version() {
    let server = start_replay(load_fixture()).await;

    let client = DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build();

    let version = client
        .daemon()
        .get_version()
        .await
        .expect("daemon.get_version");
    assert_eq!(version, "2.1.2.dev0");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_free_space_returns_bytes() {
    let server = start_replay(load_fixture()).await;

    let client = DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build();

    let space = client
        .core()
        .session
        .get_free_space(None)
        .await
        .expect("core.get_free_space");
    assert!(space > 0, "free space should be positive, got {space}");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_torrents_list_returns_entries() {
    let server = start_replay(load_fixture()).await;

    let client = DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build();

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
        .session
        .get_session_status(&[])
        .await
        .expect("core.get_session_status");

    assert!(
        status.extra.contains_key("dht.dht_nodes") || !status.extra.is_empty(),
        "session status should have overflow metrics"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_config_deserializes() {
    let server = start_replay(load_fixture()).await;

    let client = DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build();

    let config = client
        .core()
        .config
        .get_config()
        .await
        .expect("core.get_config");

    assert!(
        config.daemon_port == 58846,
        "daemon_port should be 58846, got {}",
        config.daemon_port
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_live_daemon_cassette_then_enabled_plugins_returns_list() {
    let server = start_replay(load_fixture()).await;

    let client = DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build();

    let plugins = client
        .core()
        .plugins
        .get_enabled_plugins()
        .await
        .expect("core.get_enabled_plugins");

    assert!(
        !plugins.is_empty(),
        "should have at least one enabled plugin"
    );
}
