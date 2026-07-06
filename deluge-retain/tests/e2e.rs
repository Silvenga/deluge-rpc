mod common;

use assert_cmd::Command;
use assert_fs::NamedTempFile;
use assert_fs::fixture::FileWriteStr;
use chrono::Utc;
use common::cassettes;
use deluge_rpc_mock::Matcher;
use deluge_rpc_mock::ReplayServer;
use std::time::Duration;

fn old_timestamp() -> i64 {
    Utc::now().timestamp() - 60 * 60 * 24 * 30
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
    let cassette = cassettes::torrents_list(info_hash, "old-torrent", old_timestamp());
    let matcher = Matcher::new(cassette.interactions);
    let server = ReplayServer::start(matcher)
        .await
        .expect("start replay server");
    let config = write_config(&server.host(), server.port(), "10 GiB", "20 GiB");

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

    let methods = server.consumed_methods();
    let remove_calls = methods
        .iter()
        .filter(|m| *m == "core.remove_torrent")
        .count();
    assert_eq!(
        remove_calls, 0,
        "dry run must not issue any core.remove_torrent calls, got {methods:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn when_once_live_then_calls_remove_torrent() {
    let info_hash = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111";
    let cassette = cassettes::remove_torrent(info_hash, old_timestamp());
    let matcher = Matcher::new(cassette.interactions);
    let server = ReplayServer::start(matcher)
        .await
        .expect("start replay server");
    let config = write_config(&server.host(), server.port(), "10 GiB", "20 GiB");

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

    let methods = server.consumed_methods();
    let remove_calls = methods
        .iter()
        .filter(|m| *m == "core.remove_torrent")
        .count();
    assert_eq!(
        remove_calls, 1,
        "exactly one core.remove_torrent call expected, got {methods:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn when_free_space_above_low_water_mark_then_logs_ok() {
    let cassette = cassettes::free_space_high();
    let matcher = Matcher::new(cassette.interactions);
    let server = ReplayServer::start(matcher)
        .await
        .expect("start replay server");
    let config = write_config(&server.host(), server.port(), "10 GiB", "20 GiB");

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

    let methods = server.consumed_methods();
    assert!(
        !methods.iter().any(|m| m == "core.get_torrents_status"),
        "no torrent list query when free space is above the low water mark, got {methods:?}"
    );
}
