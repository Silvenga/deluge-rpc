//! E2e tests for core plugin management RPC methods against a cassette replay server.

mod common;

const FIXTURE: &str = "plugins-manage.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_plugins_manage_cassette_then_get_available_returns_list() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let plugins = client
        .core
        .plugins
        .get_available_plugins()
        .await
        .expect("core.get_available_plugins");

    assert!(
        !plugins.is_empty(),
        "available plugins should not be empty, got {plugins:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_plugins_manage_cassette_then_enable_label_returns_true() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core
        .plugins
        .enable_plugin("Label")
        .await
        .expect("core.enable_plugin");

    assert!(result, "enable_plugin Label should return true");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_plugins_manage_cassette_then_disable_label_returns_true() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core
        .plugins
        .disable_plugin("Label")
        .await
        .expect("core.disable_plugin");

    assert!(result, "disable_plugin Label should return true");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_plugins_manage_cassette_then_rescan_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .core
        .plugins
        .rescan_plugins()
        .await
        .expect("core.rescan_plugins");
}
