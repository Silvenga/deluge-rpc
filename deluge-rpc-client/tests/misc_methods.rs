//! E2e tests for core misc RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::CoreMiscRpc;

const FIXTURE: &str = "misc-methods.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_misc_methods_cassette_then_glob_returns_list() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core()
        .misc
        .glob("/root/Downloads/*")
        .await
        .expect("core.glob");

    assert!(
        result.is_empty(),
        "glob of empty dir should return empty list"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_misc_methods_cassette_then_completion_paths_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core()
        .misc
        .get_completion_paths("/root/", false)
        .await
        .expect("core.get_completion_paths");

    assert_eq!(result.completion_text, "/root/");
    assert!(!result.show_hidden_files);
}

#[tokio::test(flavor = "multi_thread")]
async fn when_misc_methods_cassette_then_create_torrent_returns_filename_and_dump() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let result = client
        .core()
        .misc
        .create_torrent(
            "/config/testfile.txt",
            "http://example.com/announce",
            262144,
            None,
            None,
            None,
            false,
            None,
            None,
            false,
        )
        .await
        .expect("core.create_torrent");

    assert!(!result.filename.is_empty(), "filename should not be empty");
    assert!(
        !result.file_dump.is_empty(),
        "file_dump should not be empty"
    );
}
