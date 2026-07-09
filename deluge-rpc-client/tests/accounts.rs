//! E2e tests for core account management RPC methods against a cassette replay server.

mod common;

const FIXTURE: &str = "accounts.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_accounts_cassette_then_get_known_accounts_returns_list() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let accounts = client
        .core
        .accounts
        .get_known_accounts()
        .await
        .expect("core.get_known_accounts");

    assert!(
        !accounts.is_empty(),
        "should have at least one account (localclient)"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_accounts_cassette_then_get_auth_levels_mappings_returns_tuple() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let (name_to_int, int_to_name) = client
        .core
        .accounts
        .get_auth_levels_mappings()
        .await
        .expect("core.get_auth_levels_mappings");

    assert!(
        name_to_int.contains_key("ADMIN"),
        "should contain ADMIN mapping"
    );
    assert!(
        int_to_name.contains_key(&10),
        "should contain 10->ADMIN mapping"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_accounts_cassette_then_create_account_returns_true() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core
        .accounts
        .create_account("testuser", "testpass", "NORMAL")
        .await
        .expect("core.create_account");

    assert!(result, "create_account should return true");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_accounts_cassette_then_update_account_returns_true() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core
        .accounts
        .update_account("testuser", "newpass", "ADMIN")
        .await
        .expect("core.update_account");

    assert!(result, "update_account should return true");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_accounts_cassette_then_remove_account_returns_true() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core
        .accounts
        .remove_account("testuser")
        .await
        .expect("core.remove_account");

    assert!(result, "remove_account should return true");
}
