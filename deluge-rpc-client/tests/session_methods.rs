//! E2e tests for core session RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::CoreSessionRpc;

const FIXTURE: &str = "session-methods.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_session_methods_cassette_then_pause_is_paused_resume_not_paused() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .core()
        .session
        .pause_session()
        .await
        .expect("core.pause_session");

    let paused = client
        .core()
        .session
        .is_session_paused()
        .await
        .expect("core.is_session_paused after pause");
    assert!(paused, "session should be paused after pause_session");

    client
        .core()
        .session
        .resume_session()
        .await
        .expect("core.resume_session");

    let paused = client
        .core()
        .session
        .is_session_paused()
        .await
        .expect("core.is_session_paused after resume");
    assert!(!paused, "session should not be paused after resume_session");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_session_methods_cassette_then_listen_port_is_positive() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let port = client
        .core()
        .session
        .get_listen_port()
        .await
        .expect("core.get_listen_port");

    assert!(port > 0, "listen port should be positive, got {port}");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_session_methods_cassette_then_external_ip_is_non_empty() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let ip = client
        .core()
        .session
        .get_external_ip()
        .await
        .expect("core.get_external_ip");

    assert!(
        !ip.is_empty(),
        "external IP should not be empty, got '{ip}'"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_session_methods_cassette_then_libtorrent_version_is_string() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let version = client
        .core()
        .session
        .get_libtorrent_version()
        .await
        .expect("core.get_libtorrent_version");

    assert!(
        !version.is_empty(),
        "libtorrent version should not be empty, got '{version}'"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_session_methods_cassette_then_test_listen_port_returns_bool() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core()
        .session
        .test_listen_port()
        .await
        .expect("core.test_listen_port");

    assert_eq!(
        result,
        Some(false),
        "test_listen_port should return Some(false) from fixture"
    );
}
