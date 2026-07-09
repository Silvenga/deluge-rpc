//! E2e tests for blocklist plugin RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::models::BlocklistConfig;

const FIXTURE: &str = "blocklist.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_blocklist_cassette_then_get_config_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = client
        .plugins
        .blocklist
        .get_config()
        .await
        .expect("blocklist.get_config");

    assert_eq!(config.check_after_days, 4);
}

#[tokio::test(flavor = "multi_thread")]
async fn when_blocklist_cassette_then_get_status_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let status = client
        .plugins
        .blocklist
        .get_status()
        .await
        .expect("blocklist.get_status");

    assert!(!status.state.is_empty(), "state should not be empty");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_blocklist_cassette_then_set_config_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = BlocklistConfig {
        url: String::new(),
        load_on_start: true,
        check_after_days: 4,
        list_compression: String::new(),
        list_type: "text".to_owned(),
        last_update: 0.0,
        list_size: 0,
        timeout: 30,
        try_times: 3,
        whitelisted: vec![],
    };

    client
        .plugins
        .blocklist
        .set_config(&config)
        .await
        .expect("blocklist.set_config");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_blocklist_cassette_then_check_import_returns_none() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .plugins
        .blocklist
        .check_import(true)
        .await
        .expect("blocklist.check_import");

    assert!(
        result.is_none(),
        "check_import with empty URL should return None"
    );
}
