//! E2e tests for daemon RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::DaemonRpc;

const FIXTURE: &str = "daemon-methods.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_daemon_methods_cassette_then_get_method_list_returns_non_empty() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;

    let client = common::build_client(&server).await;

    let methods = client
        .daemon()
        .get_method_list()
        .await
        .expect("daemon.get_method_list");

    assert!(
        !methods.is_empty(),
        "method list should not be empty, got {methods:?}"
    );
    assert!(
        methods.iter().any(|m| m.starts_with("core.")),
        "method list should contain core.* methods, got {methods:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_daemon_methods_cassette_then_set_event_interest_returns_true() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;

    let client = common::build_client(&server).await;

    let result = client
        .daemon()
        .set_event_interest(&[
            "TorrentAddedEvent".to_owned(),
            "ConfigValueChangedEvent".to_owned(),
        ])
        .await
        .expect("daemon.set_event_interest");

    assert!(result, "set_event_interest should return true");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_daemon_methods_cassette_then_authorized_call_returns_true() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;

    let client = common::build_client(&server).await;

    let result = client
        .daemon()
        .authorized_call("core.get_config")
        .await
        .expect("daemon.authorized_call");

    assert!(
        result,
        "authorized_call for core.get_config should return true with admin auth"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_daemon_methods_cassette_then_shutdown_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;

    let client = common::build_client(&server).await;

    client
        .daemon()
        .shutdown()
        .await
        .expect("daemon.shutdown should succeed against cassette");
}
