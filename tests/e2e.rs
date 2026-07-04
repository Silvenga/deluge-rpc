//! End-to-end integration tests.
//!
//! These tests invoke the `deluge-retain` binary as a subprocess against a
//! [`MockDelugeDaemon`] speaking the native Deluge daemon RPC wire protocol.
//! They verify the observable behavior of the binary: dry-run logs the plan
//! without deleting, live mode issues `core.remove_torrent` calls, and a host
//! above the low water mark logs `OK` and makes no torrent queries.
//!
//! The binary's `tracing` subscriber writes to **stdout** (the default
//! `tracing_subscriber::fmt` layer target), so assertions are on stdout.

#![expect(
    clippy::expect_used,
    reason = "integration tests panic on unexpected shapes via expect for clarity"
)]
#![expect(
    clippy::unwrap_used,
    reason = "integration tests use unwrap for clarity on known-good values"
)]

mod common;

use std::collections::BTreeMap;
use std::time::Duration;

use assert_cmd::Command;
use assert_fs::NamedTempFile;
use assert_fs::fixture::FileWriteStr;

use chrono::Utc;
use common::mock_daemon::{MockDaemonConfig, MockDelugeDaemon, MockResponse};
use deluge_retain::rencode::RencodeValue;

const GB: u64 = 1_073_741_824;

fn old_timestamp() -> i64 {
    Utc::now().timestamp() - 60 * 60 * 24 * 30
}

fn torrents_dict(info_hash: &str, name: &str, ratio: f64, total_done: u64, time_added: i64) -> RencodeValue {
    let mut fields = BTreeMap::new();
    fields.insert(RencodeValue::Str(String::from("name")), RencodeValue::Str(String::from(name)));
    fields.insert(RencodeValue::Str(String::from("state")), RencodeValue::Str(String::from("Seeding")));
    fields.insert(RencodeValue::Str(String::from("progress")), RencodeValue::Float(100.0));
    fields.insert(RencodeValue::Str(String::from("ratio")), RencodeValue::Float(ratio));
    fields.insert(RencodeValue::Str(String::from("total_seeds")), RencodeValue::Int(50));
    fields.insert(RencodeValue::Str(String::from("num_seeds")), RencodeValue::Int(5));
    fields.insert(RencodeValue::Str(String::from("time_added")), RencodeValue::Int(time_added));
    fields.insert(
        RencodeValue::Str(String::from("total_done")),
        RencodeValue::Int(i64::try_from(total_done).unwrap()),
    );
    fields.insert(RencodeValue::Str(String::from("total_uploaded")), RencodeValue::Int(0));
    fields.insert(RencodeValue::Str(String::from("is_finished")), RencodeValue::Bool(true));
    fields.insert(
        RencodeValue::Str(String::from("download_location")),
        RencodeValue::Str(String::from("/data")),
    );

    let mut dict = BTreeMap::new();
    dict.insert(RencodeValue::Str(String::from(info_hash)), RencodeValue::Dict(fields));
    RencodeValue::Dict(dict)
}

fn write_config(host: &str, port: u16, low: &str, high: &str) -> NamedTempFile {
    let contents = format!(
        r#"
poll_interval = "60s"

[[hosts]]
host = "{host}"
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn when_once_dry_run_then_logs_plan_and_makes_no_remove_calls() {
    let info_hash = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111";
    let config_mock = MockDaemonConfig {
        login: MockResponse::success(RencodeValue::Int(5)),
        free_space: MockResponse::success(RencodeValue::Int(i64::try_from(5 * GB).unwrap())),
        torrents: MockResponse::success(torrents_dict(info_hash, "old-torrent", 3.0, 2 * GB, old_timestamp())),
        remove: MockResponse::success(RencodeValue::Bool(true)),
    };
    let mock = MockDelugeDaemon::start(config_mock).await;
    let config = write_config(&mock.host(), mock.port(), "10 GiB", "20 GiB");

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

    let methods = mock.received_methods();
    let remove_calls = methods.iter().filter(|m| *m == "core.remove_torrent").count();
    assert_eq!(remove_calls, 0, "dry run must not issue any core.remove_torrent calls, got {methods:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn when_once_live_then_calls_remove_torrent() {
    let info_hash = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111";
    let config_mock = MockDaemonConfig {
        login: MockResponse::success(RencodeValue::Int(5)),
        free_space: MockResponse::success(RencodeValue::Int(i64::try_from(5 * GB).unwrap())),
        torrents: MockResponse::success(torrents_dict(info_hash, "old-torrent", 3.0, 2 * GB, old_timestamp())),
        remove: MockResponse::success(RencodeValue::Bool(true)),
    };
    let mock = MockDelugeDaemon::start(config_mock).await;
    let config = write_config(&mock.host(), mock.port(), "10 GiB", "20 GiB");

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

    let methods = mock.received_methods();
    let remove_calls = methods.iter().filter(|m| *m == "core.remove_torrent").count();
    assert_eq!(remove_calls, 1, "exactly one core.remove_torrent call expected, got {methods:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn when_free_space_above_low_water_mark_then_logs_ok() {
    let config_mock = MockDaemonConfig {
        login: MockResponse::success(RencodeValue::Int(5)),
        free_space: MockResponse::success(RencodeValue::Int(i64::try_from(30 * GB).unwrap())),
        torrents: MockResponse::NotConfigured,
        remove: MockResponse::NotConfigured,
    };
    let mock = MockDelugeDaemon::start(config_mock).await;
    let config = write_config(&mock.host(), mock.port(), "10 GiB", "20 GiB");

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

    let methods = mock.received_methods();
    assert!(
        !methods.iter().any(|m| m == "core.get_torrents_status"),
        "no torrent list query when free space is above the low water mark, got {methods:?}"
    );
}
