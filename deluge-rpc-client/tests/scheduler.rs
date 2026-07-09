//! E2e tests for scheduler plugin RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::SchedulerRpc;
use deluge_rpc_client::models::{SchedulerConfig, SchedulerState};

const FIXTURE: &str = "scheduler.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_scheduler_cassette_then_get_config_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = client
        .plugins()
        .scheduler
        .get_config()
        .await
        .expect("scheduler.get_config");

    assert_eq!(
        config.button_state.len(),
        24,
        "button_state should be 24 hours"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_scheduler_cassette_then_get_state_returns_green() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let state = client
        .plugins()
        .scheduler
        .get_state()
        .await
        .expect("scheduler.get_state");

    assert_eq!(state, SchedulerState::Green);
}

#[tokio::test(flavor = "multi_thread")]
async fn when_scheduler_cassette_then_set_config_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = SchedulerConfig {
        low_down: -1.0,
        low_up: -1.0,
        low_active: -1,
        low_active_down: -1,
        low_active_up: -1,
        button_state: vec![vec![0, 0, 0, 0, 0, 0, 0]; 24],
    };

    client
        .plugins()
        .scheduler
        .set_config(&config)
        .await
        .expect("scheduler.set_config");
}
