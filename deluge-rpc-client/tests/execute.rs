//! E2e tests for execute plugin RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::ExecuteRpc;
use deluge_rpc_client::models::ExecuteEvent;

const FIXTURE: &str = "execute.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_execute_cassette_then_get_commands_returns_list() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let commands = client
        .plugins()
        .execute
        .get_commands()
        .await
        .expect("execute.get_commands");

    assert!(
        commands.is_empty(),
        "commands should be empty on fresh daemon"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_execute_cassette_then_add_command_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .plugins()
        .execute
        .add_command(&ExecuteEvent::Complete, "echo done")
        .await
        .expect("execute.add_command");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_execute_cassette_then_save_command_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .plugins()
        .execute
        .save_command("abc123", &ExecuteEvent::Added, "echo hello")
        .await
        .expect("execute.save_command");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_execute_cassette_then_remove_command_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .plugins()
        .execute
        .remove_command("abc123")
        .await
        .expect("execute.remove_command");
}
