//! E2e tests for notifications plugin RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::NotificationsRpc;
use deluge_rpc_client::models::NotificationsConfig;

const FIXTURE: &str = "notifications.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_notifications_cassette_then_get_config_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = client
        .plugins()
        .notifications
        .get_config()
        .await
        .expect("notifications.get_config");

    assert!(
        !config.smtp_enabled,
        "smtp_enabled should be false by default"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_notifications_cassette_then_get_handled_events_returns_list() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let events = client
        .plugins()
        .notifications
        .get_handled_events()
        .await
        .expect("notifications.get_handled_events");

    assert!(!events.is_empty(), "handled events should not be empty");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_notifications_cassette_then_set_config_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = NotificationsConfig {
        smtp_enabled: false,
        smtp_host: String::new(),
        smtp_port: 25,
        smtp_user: String::new(),
        smtp_pass: String::new(),
        smtp_from: String::new(),
        smtp_tls: false,
        smtp_recipients: vec![],
        subscriptions: Default::default(),
    };

    client
        .plugins()
        .notifications
        .set_config(&config)
        .await
        .expect("notifications.set_config");
}
