//! E2e tests for webui plugin RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::WebUiRpc;
use deluge_rpc_client::models::WebUiConfig;

const FIXTURE: &str = "webui.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_webui_cassette_then_get_config_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = client
        .plugins()
        .webui
        .get_config()
        .await
        .expect("webui.get_config");

    assert!(!config.enabled, "webui should be disabled by default");
    assert_eq!(config.port, 8112, "port should be 8112");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_webui_cassette_then_got_deluge_web_returns_bool() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .plugins()
        .webui
        .got_deluge_web()
        .await
        .expect("webui.got_deluge_web");

    assert!(
        result,
        "got_deluge_web should return true if deluge-web is installed"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_webui_cassette_then_set_config_returns_error_for_port_in_use() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = WebUiConfig {
        enabled: false,
        ssl: false,
        port: 8112,
    };

    let result = client.plugins().webui.set_config(&config).await;
    assert!(
        result.is_err(),
        "set_config should fail when port is already in use"
    );
}
