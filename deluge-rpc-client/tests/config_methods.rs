//! E2e tests for core config RPC methods against a cassette replay server.

mod common;

use std::collections::BTreeMap;

const FIXTURE: &str = "config-methods.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_config_methods_cassette_then_get_config_value_returns_path() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let value = client
        .core
        .config
        .get_config_value("download_location")
        .await
        .expect("core.get_config_value");

    match value {
        deluge_rpc_client::RencodeValue::Str(s) => {
            assert!(!s.is_empty(), "download_location should not be empty");
        }
        other => panic!("expected str, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn when_config_methods_cassette_then_get_config_values_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let values = client
        .core
        .config
        .get_config_values(&["download_location".to_owned(), "daemon_port".to_owned()])
        .await
        .expect("core.get_config_values");

    assert!(
        values.contains_key("download_location"),
        "should contain download_location, got {values:?}"
    );
    assert!(
        values.contains_key("daemon_port"),
        "should contain daemon_port, got {values:?}"
    );
    assert_eq!(
        values.get("daemon_port"),
        Some(&deluge_rpc_client::RencodeValue::Int(58846)),
        "daemon_port should be 58846"
    );
    assert_eq!(
        values.get("download_location"),
        Some(&deluge_rpc_client::RencodeValue::Str(
            "/root/Downloads".to_owned()
        )),
        "download_location should be /root/Downloads"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_config_methods_cassette_then_set_config_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let mut config = BTreeMap::new();
    config.insert(
        "send_info".to_owned(),
        deluge_rpc_client::RencodeValue::Bool(false),
    );

    client
        .core
        .config
        .set_config(&config)
        .await
        .expect("core.set_config");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_config_methods_cassette_then_get_proxy_returns_config() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let proxy = client
        .core
        .config
        .get_proxy()
        .await
        .expect("core.get_proxy");

    assert_eq!(proxy.proxy_type, 0, "proxy type should be 0 (None)");
}
