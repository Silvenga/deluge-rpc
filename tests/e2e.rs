//! End-to-end integration tests.
//!
//! These tests invoke the `deluge-retain` binary as a subprocess against a
//! `wiremock` mock of a full Deluge Web UI JSON-RPC server. They verify the
//! observable behavior of the binary: dry-run logs the plan without deleting,
//! live mode issues `core.remove_torrent` calls, and a host above the low
//! water mark logs `OK` and makes no torrent queries.
//!
//! The binary's `tracing` subscriber writes to **stdout** (the default
//! `tracing_subscriber::fmt` layer target), so assertions are on stdout.

#![expect(
    clippy::expect_used,
    reason = "integration tests panic on unexpected shapes via expect for clarity"
)]

use std::time::Duration;

use assert_cmd::Command;
use assert_fs::NamedTempFile;
use assert_fs::fixture::FileWriteStr;
use serde_json::{Value, json};
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const GB: u64 = 1_073_741_824;

/// A timestamp far enough in the past to satisfy `min_age_days = 1`.
fn old_timestamp() -> f64 {
    let now = chrono::Utc::now().timestamp();
    #[expect(
        clippy::as_conversions,
        reason = "i64 to f64 for the Deluge time_added field is intentional"
    )]
    #[expect(
        clippy::cast_precision_loss,
        reason = "epoch seconds fit exactly in f64 mantissa for realistic timestamps"
    )]
    {
        now as f64 - 60.0 * 60.0 * 24.0 * 30.0
    }
}

fn torrent_entry(name: &str, ratio: f64, total_done: u64, time_added: f64) -> Value {
    json!({
        "name": name,
        "state": "Seeding",
        "progress": 100.0,
        "ratio": ratio,
        "total_seeds": 50,
        "num_seeds": 5,
        "time_added": time_added,
        "total_done": total_done,
        "total_uploaded": 0,
        "is_finished": true,
        "download_location": "/data"
    })
}

/// Write a TOML config file pointing a single host at `server_uri/json`.
fn write_config(server_uri: &str, low: &str, high: &str) -> NamedTempFile {
    // server_uri looks like "http://127.0.0.1:12345"
    let no_scheme = server_uri.strip_prefix("http://").unwrap_or(server_uri);
    let (host_addr, port_str) = no_scheme
        .split_once(':')
        .expect("mock server uri has host:port");
    let port: u16 = port_str.parse().expect("mock server port is u16");
    let contents = format!(
        r#"
poll_interval = "60s"

[[hosts]]
host = "{host_addr}"
port = {port}
username = "localclient"
password = "secret"

[rules]
min_seeders = 1
min_age_days = 1
low_water_mark = "{low}"
high_water_mark = "{high}"
delete_throttle_secs = 0
"#
    );
    let file = NamedTempFile::new(".toml").expect("create temp config");
    file.write_str(&contents).expect("write temp config");
    file
}

async fn mount_login(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/json"))
        .and(body_partial_json(json!({
            "method": "auth.login",
            "params": ["secret"],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": true,
            "error": null,
            "id": 1,
        })))
        .mount(server)
        .await;
}

async fn mount_free_space(server: &MockServer, bytes: u64) {
    Mock::given(method("POST"))
        .and(path("/json"))
        .and(body_partial_json(json!({
            "method": "core.get_free_space",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": bytes,
            "error": null,
            "id": 1,
        })))
        .mount(server)
        .await;
}

async fn mount_torrents(server: &MockServer, torrents: Value) {
    Mock::given(method("POST"))
        .and(path("/json"))
        .and(body_partial_json(json!({
            "method": "web.update_ui",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": { "torrents": torrents },
            "error": null,
            "id": 1,
        })))
        .mount(server)
        .await;
}

async fn mount_remove(server: &MockServer, info_hash: &str) {
    Mock::given(method("POST"))
        .and(path("/json"))
        .and(body_partial_json(json!({
            "method": "core.remove_torrent",
            "params": [info_hash, true],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": true,
            "error": null,
            "id": 1,
        })))
        .mount(server)
        .await;
}

/// Count requests received by the mock server whose JSON-RPC `method`
/// matches `method_name`.
async fn count_calls(server: &MockServer, method_name: &str) -> usize {
    let received = server.received_requests().await.expect("read requests");
    received
        .iter()
        .filter(|r| {
            let body: Value = serde_json::from_slice(&r.body).expect("body is json");
            body.get("method").and_then(|m| m.as_str()) == Some(method_name)
        })
        .count()
}

/// Test 1: `--once --dry-run` against a mock server with free space below
/// the low water mark. The binary must log the dry-run plan and make zero
/// `core.remove_torrent` calls.
#[tokio::test]
#[ignore = "task-8: rewrite against daemon RPC mock"]
async fn when_once_dry_run_then_logs_plan_and_makes_no_remove_calls_should_skip_deletion() {
    let server = MockServer::start().await;
    mount_login(&server).await;
    mount_free_space(&server, 5 * GB).await;
    let info_hash = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111";
    let torrents = json!({
        info_hash: torrent_entry("old-torrent", 3.0, 2 * GB, old_timestamp()),
    });
    mount_torrents(&server, torrents).await;

    let config = write_config(&server.uri(), "10 GiB", "20 GiB");

    let output = Command::cargo_bin("deluge-retain")
        .expect("find deluge-retain binary")
        .args([
            "--config",
            config.to_str().expect("config path is utf-8"),
            "--once",
            "--dry-run",
        ])
        .timeout(Duration::from_secs(30))
        .output()
        .expect("run binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "binary should exit 0, got {:?}\nstdout:\n{stdout}",
        output.status.code()
    );

    assert!(
        stdout.contains("DRY RUN") || stdout.contains("would delete"),
        "dry-run output should mention DRY RUN or 'would delete', got stdout:\n{stdout}"
    );

    let remove_calls = count_calls(&server, "core.remove_torrent").await;
    assert_eq!(
        remove_calls, 0,
        "dry run must not issue any core.remove_torrent calls"
    );
}

/// Test 2: `--once` (live mode) against a mock server with free space below
/// the low water mark. The binary must issue `core.remove_torrent` calls for
/// the planned torrents.
#[tokio::test]
#[ignore = "task-8: rewrite against daemon RPC mock"]
async fn when_once_live_then_calls_remove_torrent_should_delete() {
    let server = MockServer::start().await;
    mount_login(&server).await;
    mount_free_space(&server, 5 * GB).await;
    let info_hash = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111";
    let torrents = json!({
        info_hash: torrent_entry("old-torrent", 3.0, 2 * GB, old_timestamp()),
    });
    mount_torrents(&server, torrents).await;
    mount_remove(&server, info_hash).await;

    let config = write_config(&server.uri(), "10 GiB", "20 GiB");

    let output = Command::cargo_bin("deluge-retain")
        .expect("find deluge-retain binary")
        .args([
            "--config",
            config.to_str().expect("config path is utf-8"),
            "--once",
        ])
        .timeout(Duration::from_secs(30))
        .output()
        .expect("run binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "binary should exit 0, got {:?}\nstdout:\n{stdout}",
        output.status.code()
    );

    let remove_calls = count_calls(&server, "core.remove_torrent").await;
    assert_eq!(
        remove_calls, 1,
        "exactly one core.remove_torrent call expected for the planned torrent"
    );
}

/// Test 3: free space above the low water mark. The binary must log `OK`
/// and make no `web.update_ui` calls.
#[tokio::test]
#[ignore = "task-8: rewrite against daemon RPC mock"]
async fn when_free_space_above_low_water_mark_then_logs_ok_and_no_torrent_query() {
    let server = MockServer::start().await;
    mount_login(&server).await;
    mount_free_space(&server, 30 * GB).await;

    let config = write_config(&server.uri(), "10 GiB", "20 GiB");

    let output = Command::cargo_bin("deluge-retain")
        .expect("find deluge-retain binary")
        .args([
            "--config",
            config.to_str().expect("config path is utf-8"),
            "--once",
        ])
        .timeout(Duration::from_secs(30))
        .output()
        .expect("run binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "binary should exit 0, got {:?}\nstdout:\n{stdout}",
        output.status.code()
    );

    assert!(
        stdout.contains("OK"),
        "output should log OK when free space is above the low water mark, got stdout:\n{stdout}"
    );

    let update_ui_calls = count_calls(&server, "web.update_ui").await;
    assert_eq!(
        update_ui_calls, 0,
        "no torrent list query when free space is above the low water mark"
    );
}
