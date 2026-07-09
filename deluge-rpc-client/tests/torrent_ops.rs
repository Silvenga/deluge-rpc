//! E2e tests for torrent operation RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::CoreTorrentRpc;
use deluge_rpc_client::models::{SetTorrentOptions, TrackerInfo};

const FIXTURE: &str = "torrent-ops.json";
const TORRENT_ID: &str = "38645727754424361ddcc6e14d974eac9e7c6456";

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_get_filter_tree_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core()
        .torrents
        .get_filter_tree(true, None)
        .await
        .expect("core.get_filter_tree");

    assert!(!result.is_empty(), "filter tree should not be empty");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_get_session_state_returns_list() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core()
        .torrents
        .get_session_state()
        .await
        .expect("core.get_session_state");

    assert!(
        !result.is_empty(),
        "session state should have at least one torrent"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_get_path_size_returns_positive() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core()
        .torrents
        .get_path_size("/config")
        .await
        .expect("core.get_path_size");

    assert!(
        result >= 0,
        "path size should be non-negative, got {result}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_pause_and_resume_torrent_succeed() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .core()
        .torrents
        .pause_torrent(TORRENT_ID)
        .await
        .expect("core.pause_torrent");

    client
        .core()
        .torrents
        .resume_torrent(TORRENT_ID)
        .await
        .expect("core.resume_torrent");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_pause_all_and_resume_all_succeed() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .core()
        .torrents
        .pause_torrents(None)
        .await
        .expect("core.pause_torrents");

    client
        .core()
        .torrents
        .resume_torrents(None)
        .await
        .expect("core.resume_torrents");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_force_reannounce_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .core()
        .torrents
        .force_reannounce(&[TORRENT_ID.to_owned()])
        .await
        .expect("core.force_reannounce");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_force_recheck_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .core()
        .torrents
        .force_recheck(&[TORRENT_ID.to_owned()])
        .await
        .expect("core.force_recheck");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_set_torrent_options_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let options = SetTorrentOptions {
        max_connections: Some(10),
        ..Default::default()
    };

    client
        .core()
        .torrents
        .set_torrent_options(&[TORRENT_ID.to_owned()], &options)
        .await
        .expect("core.set_torrent_options");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_connect_peer_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .core()
        .torrents
        .connect_peer(TORRENT_ID, "127.0.0.1", 6881)
        .await
        .expect("core.connect_peer");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_move_storage_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .core()
        .torrents
        .move_storage(&[TORRENT_ID.to_owned()], "/tmp")
        .await
        .expect("core.move_storage");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_set_ssl_cert_returns_error() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core()
        .torrents
        .set_ssl_torrent_cert(TORRENT_ID, "cert", "key", "dh", true)
        .await;

    assert!(
        result.is_err(),
        "set_ssl_torrent_cert should return error for dummy certs"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_set_trackers_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let trackers = vec![TrackerInfo {
        url: "http://example.com/announce".to_owned(),
        tier: 0,
    }];

    client
        .core()
        .torrents
        .set_torrent_trackers(TORRENT_ID, &trackers)
        .await
        .expect("core.set_torrent_trackers");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_rename_files_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .core()
        .torrents
        .rename_files(TORRENT_ID, &[(0, "renamed.txt".to_owned())])
        .await
        .expect("core.rename_files");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_rename_folder_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    client
        .core()
        .torrents
        .rename_folder(TORRENT_ID, "old", "new")
        .await
        .expect("core.rename_folder");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_ops_cassette_then_queue_ops_succeed() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;
    let ids = [TORRENT_ID.to_owned()];

    client
        .core()
        .torrents
        .queue_top(&ids)
        .await
        .expect("core.queue_top");
    client
        .core()
        .torrents
        .queue_up(&ids)
        .await
        .expect("core.queue_up");
    client
        .core()
        .torrents
        .queue_down(&ids)
        .await
        .expect("core.queue_down");
    client
        .core()
        .torrents
        .queue_bottom(&ids)
        .await
        .expect("core.queue_bottom");
}
