//! E2e tests for label plugin RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::models::{LabelConfig, LabelOptions};

const FIXTURE: &str = "label.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_label_cassette_then_get_labels_returns_list() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let labels = client
        .plugins
        .label
        .get_labels()
        .await
        .expect("label.get_labels");

    assert!(labels.is_empty(), "labels should be empty on fresh daemon");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_label_cassette_then_add_label_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .plugins
        .label
        .add("test-label")
        .await
        .expect("label.add");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_label_cassette_then_get_options_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let options = client
        .plugins
        .label
        .get_options("test-label")
        .await
        .expect("label.get_options");

    assert!(!options.apply_max, "apply_max should default to false");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_label_cassette_then_set_options_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let options = LabelOptions {
        apply_max: false,
        max_download_speed: None,
        max_upload_speed: None,
        max_connections: None,
        max_upload_slots: None,
        prioritize_first_last: false,
        apply_queue: false,
        is_auto_managed: false,
        stop_at_ratio: false,
        stop_ratio: 2.0,
        remove_at_ratio: false,
        apply_move_completed: false,
        move_completed: false,
        move_completed_path: String::new(),
        auto_add: false,
        auto_add_trackers: vec![],
    };

    client
        .plugins
        .label
        .set_options("test-label", &options)
        .await
        .expect("label.set_options");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_label_cassette_then_get_config_succeeds_with_defaults() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config =
        client.plugins.label.get_config().await.expect(
            "label.get_config should succeed with serde default for missing auto_add_trackers",
        );

    assert!(
        config.auto_add_trackers.is_empty(),
        "auto_add_trackers should default to empty when daemon omits the field"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_label_cassette_then_set_config_returns_error() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = LabelConfig {
        auto_add_trackers: vec![],
    };

    let result = client.plugins.label.set_config(&config).await;
    assert!(result.is_err(), "set_config should fail against cassette");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_label_cassette_then_set_torrent_returns_error() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .plugins
        .label
        .set_torrent("83d40fa6191f96716d36bee7bc04274ee792ec45", "test-label")
        .await;
    assert!(
        result.is_err(),
        "set_torrent should fail for non-existent torrent"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_label_cassette_then_remove_label_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .plugins
        .label
        .remove("test-label")
        .await
        .expect("label.remove");
}
