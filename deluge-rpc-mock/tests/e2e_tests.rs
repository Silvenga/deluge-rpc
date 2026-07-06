use deluge_rpc::{DaemonRpc, DelugeClientBuilder, RencodeValue};
use deluge_rpc_mock::{
    Cassette, Interaction, InteractionRequest, InteractionResponse as CassetteResponse, Matcher,
    ReplayServer,
};

fn empty_cassette() -> Cassette {
    Cassette {
        version: 1,
        recorded_at: "2026-07-04T12:00:00Z".into(),
        daemon_version: None,
        interactions: vec![],
    }
}

fn ok_interaction(method: &str, value: &str) -> Interaction {
    Interaction {
        request: InteractionRequest {
            method: method.into(),
            args: RencodeValue::List(vec![]),
            kwargs: RencodeValue::List(vec![]),
        },
        response: CassetteResponse::Ok {
            value: RencodeValue::Str(value.into()),
        },
    }
}

async fn start(cassette: Cassette) -> ReplayServer {
    let matcher = Matcher::new(cassette.interactions);
    ReplayServer::start(matcher)
        .await
        .expect("start replay server")
}

#[tokio::test(flavor = "multi_thread")]
async fn when_login_then_auto_served_response_lets_connect_succeed() {
    let server = start(empty_cassette()).await;

    let _client = DelugeClientBuilder::new(
        server.host(),
        server.port(),
        "any".to_owned(),
        "any".to_owned(),
    )
    .build();
}

#[tokio::test(flavor = "multi_thread")]
async fn when_method_not_in_cassette_then_returns_unknown_method_error() {
    let server = start(empty_cassette()).await;

    let client = DelugeClientBuilder::new(server.host(), server.port(), "any".to_owned(), "any".to_owned())
        .build();

    let result = client.daemon().info().await;
    assert!(result.is_err(), "unknown method should error: {result:?}");
    let msg = format!("{result:?}");
    assert!(
        msg.contains("UnknownMethod"),
        "error should mention UnknownMethod, got: {msg}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn when_cassette_has_interaction_then_response_replayed_to_client() {
    let cassette = Cassette {
        version: 1,
        recorded_at: "2026-07-04T12:00:00Z".into(),
        daemon_version: None,
        interactions: vec![ok_interaction("daemon.info", "2.1.1")],
    };
    let server = start(cassette).await;

    let client = DelugeClientBuilder::new(server.host(), server.port(), "any".to_owned(), "any".to_owned())
        .build();

    let info = client.daemon().info().await.expect("daemon.info");
    assert_eq!(info, "2.1.1");
}
