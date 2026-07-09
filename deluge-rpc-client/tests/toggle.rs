//! E2e tests for toggle plugin RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::ToggleRpc;

const FIXTURE: &str = "toggle.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_toggle_cassette_then_get_status_returns_false() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let status = client
        .plugins()
        .toggle
        .get_status()
        .await
        .expect("toggle.get_status");

    assert!(!status, "session should not be paused initially");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_toggle_cassette_then_toggle_returns_true() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .plugins()
        .toggle
        .toggle()
        .await
        .expect("toggle.toggle");

    assert!(
        result,
        "toggle should return true (new paused state) after toggling"
    );
}
