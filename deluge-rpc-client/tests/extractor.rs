//! E2e tests for extractor plugin RPC methods against a cassette replay server.

mod common;

use deluge_rpc_client::models::ExtractorConfig;

const FIXTURE: &str = "extractor.json";

#[tokio::test(flavor = "multi_thread")]
async fn when_extractor_cassette_then_get_config_returns_dict() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = client
        .plugins
        .extractor
        .get_config()
        .await
        .expect("extractor.get_config");

    assert!(
        config.use_name_folder,
        "use_name_folder should default to true"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_extractor_cassette_then_set_config_succeeds() {
    let server = common::start_replay(common::load_fixture(FIXTURE)).await;
    let client = common::build_client(&server).await;

    let config = ExtractorConfig {
        extract_path: String::new(),
        use_name_folder: true,
    };

    client
        .plugins
        .extractor
        .set_config(&config)
        .await
        .expect("extractor.set_config");
}
