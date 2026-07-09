//! E2e tests for stats plugin RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::StatsRpc;
use deluge_rpc_client::models::StatsConfig;

const FIXTURE: &str = "stats.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_stats_cassette_then_get_totals_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let totals = client
        .plugins()
        .stats
        .get_totals()
        .await
        .expect("stats.get_totals");

    assert!(totals.total_upload >= 0, "total_upload should be >= 0");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_stats_cassette_then_get_session_totals_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let totals = client
        .plugins()
        .stats
        .get_session_totals()
        .await
        .expect("stats.get_session_totals");

    assert!(totals.total_download >= 0, "total_download should be >= 0");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_stats_cassette_then_get_intervals_returns_list() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let intervals = client
        .plugins()
        .stats
        .get_intervals()
        .await
        .expect("stats.get_intervals");

    assert_eq!(
        intervals,
        vec![1, 5, 30, 300],
        "intervals should be [1, 5, 30, 300]"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_stats_cassette_then_get_config_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = client
        .plugins()
        .stats
        .get_config()
        .await
        .expect("stats.get_config");

    assert_eq!(config.update_interval, 1, "update_interval should be 1");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_stats_cassette_then_set_config_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = StatsConfig {
        test: "NiNiNi".to_owned(),
        update_interval: 1,
        length: 150,
    };

    client
        .plugins()
        .stats
        .set_config(&config)
        .await
        .expect("stats.set_config");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_stats_cassette_then_get_stats_returns_result() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .plugins()
        .stats
        .get_stats(&["upload_rate".to_owned(), "download_rate".to_owned()], 1)
        .await
        .expect("stats.get_stats")
        .expect("get_stats should return Some for valid interval");

    assert_eq!(result.length, 150, "length should be 150");
    assert_eq!(result.update_interval, 1, "update_interval should be 1");
    assert!(
        result.stats.contains_key("upload_rate"),
        "stats should contain upload_rate"
    );
    assert!(
        result.stats.contains_key("download_rate"),
        "stats should contain download_rate"
    );
}
