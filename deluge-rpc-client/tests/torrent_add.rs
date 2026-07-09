//! E2e tests for torrent add/remove RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::CoreTorrentRpc;
use deluge_rpc_client::models::AddTorrentOptions;

const FIXTURE: &str = "torrent-add.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_add_cassette_then_add_magnet_returns_id() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let options = AddTorrentOptions::default();
    let result = client
        .core()
        .torrents
        .add_torrent_magnet(
            "magnet:?xt=urn:btih:83d40fa6191f96716d36bee7bc04274ee792ec45&dn=test",
            &options,
        )
        .await
        .expect("core.add_torrent_magnet");

    assert!(
        !result.is_empty(),
        "add_torrent_magnet should return a torrent_id"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_add_cassette_then_add_file_returns_id() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let options = AddTorrentOptions::default();
    let result = client
        .core()
        .torrents
        .add_torrent_file("test.torrent", "ZGFrZQ==", &options)
        .await
        .expect("core.add_torrent_file");

    assert!(result.is_some(), "add_torrent_file should return Some(id)");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_add_cassette_then_add_file_async_returns_error_for_duplicate() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let options = AddTorrentOptions::default();
    let result = client
        .core()
        .torrents
        .add_torrent_file_async("test2.torrent", "ZGFrZQ==", &options, true)
        .await;

    assert!(
        result.is_err(),
        "add_torrent_file_async should return error for duplicate torrent"
    );
    match result {
        Err(deluge_rpc_client::DelugeRpcError::RpcError { exc_type, .. }) => {
            assert_eq!(exc_type, "AddTorrentError");
        }
        other => panic!("expected RpcError, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_add_cassette_then_add_files_returns_error_for_duplicate() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let torrent_files = vec![(
        "test3.torrent".to_owned(),
        "ZGFrZQ==".to_owned(),
        AddTorrentOptions::default(),
    )];
    let result = client
        .core()
        .torrents
        .add_torrent_files(&torrent_files)
        .await;

    assert!(
        result.is_err(),
        "add_torrent_files should return error for duplicate torrent"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_add_cassette_then_add_url_returns_id() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let options = AddTorrentOptions::default();
    let result = client
        .core()
        .torrents
        .add_torrent_url(
            "http://releases.ubuntu.com/24.04/ubuntu-24.04.3-desktop-amd64.iso.torrent",
            &options,
            None,
        )
        .await
        .expect("core.add_torrent_url");

    assert!(result.is_some(), "add_torrent_url should return Some(id)");
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_add_cassette_then_prefetch_returns_error_for_invalid_magnet() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core()
        .torrents
        .prefetch_magnet_metadata(
            "magnet:?xt=urn:btih:83d40fa6191f96716d36bee7bc04274ee792ec45&dn=test",
            Some(10),
        )
        .await;

    assert!(
        result.is_err(),
        "prefetch_magnet_metadata should return error for invalid magnet hash"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_torrent_add_cassette_then_remove_torrents_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let ids = vec![
        "83d40fa6191f96716d36bee7bc04274ee792ec45".to_owned(),
        "38645727754424361ddcc6e14d974eac9e7c6456".to_owned(),
    ];
    let result = client
        .core()
        .torrents
        .remove_torrents(&ids, true)
        .await
        .expect("core.remove_torrents");

    assert!(
        result.is_empty(),
        "remove_torrents should return empty errors list on success"
    );
}
