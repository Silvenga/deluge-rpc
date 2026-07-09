//! E2e tests for autoadd plugin RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::models::{AutoAddConfig, WatchDirOptions};

const FIXTURE: &str = "autoadd.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_autoadd_cassette_then_get_config_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = client
        .plugins
        .auto_add
        .get_config()
        .await
        .expect("autoadd.get_config");

    assert!(config.next_id >= 1, "next_id should be >= 1");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_autoadd_cassette_then_get_watchdirs_returns_map() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let dirs = client
        .plugins
        .auto_add
        .get_watch_dirs()
        .await
        .expect("autoadd.get_watchdirs");

    assert!(dirs.is_empty(), "watchdirs should be empty on fresh daemon");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_autoadd_cassette_then_is_admin_returns_true() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .plugins
        .auto_add
        .is_admin_level()
        .await
        .expect("autoadd.is_admin_level");

    assert!(
        result,
        "is_admin_level should return true for admin session"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_autoadd_cassette_then_get_auth_user_returns_string() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let user = client
        .plugins
        .auto_add
        .get_auth_user()
        .await
        .expect("autoadd.get_auth_user");

    assert!(!user.is_empty(), "auth user should not be empty");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_autoadd_cassette_then_set_config_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = AutoAddConfig {
        watch_dirs: Default::default(),
        next_id: 1,
    };

    client
        .plugins
        .auto_add
        .set_config(&config)
        .await
        .expect("autoadd.set_config");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_autoadd_cassette_then_enable_watchdir_returns_error_for_invalid_id() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client.plugins.auto_add.enable_watch_dir(1).await;
    assert!(
        result.is_err(),
        "enable_watchdir should fail for non-existent watchdir"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_autoadd_cassette_then_disable_watchdir_returns_error_for_invalid_id() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client.plugins.auto_add.disable_watch_dir(1).await;
    assert!(
        result.is_err(),
        "disable_watchdir should fail for non-existent watchdir"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_autoadd_cassette_then_remove_returns_error_for_invalid_id() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client.plugins.auto_add.remove(1).await;
    assert!(
        result.is_err(),
        "remove should fail for non-existent watchdir"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_autoadd_cassette_then_add_returns_error() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client.plugins.auto_add.add(None).await;
    assert!(result.is_err(), "add should fail against cassette");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_autoadd_cassette_then_set_options_returns_error() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let options = WatchDirOptions {
        path: "/tmp".to_owned(),
        ..Default::default()
    };

    let result = client.plugins.auto_add.set_options(1, &options).await;
    assert!(
        result.is_err(),
        "set_options should fail for non-existent watchdir"
    );
}
