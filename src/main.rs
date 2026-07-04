//! Entry point for the deluge-retain binary.
//!
//! Wires together the CLI parser, tracing init, config loader, and the
//! retention engine into the two operating modes:
//!
//! - **once mode** (`--once`): run a single retention cycle and exit.
//! - **watch mode** (default): poll every host on `config.poll_interval`,
//!   handling `SIGINT`/`SIGTERM` for graceful shutdown between cycles.

mod cli;
mod client;
mod config;
mod engine;
mod rencode;
mod torrent;
mod tracing_setup;
mod transport;

#[cfg(test)]
mod mock_daemon;

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::Result;
use bytesize::ByteSize;
use chrono::Utc;
use clap::Parser;
use tokio::signal::unix::SignalKind;
use tokio::signal::{ctrl_c, unix};
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::cli::Cli;
use crate::client::DelugeClient;
use crate::config::{Config, HostConfig};
use crate::engine::{compute_deletion_plan, execute_deletion_plan};
use crate::tracing_setup::init_tracing;

/// Entry point for the deluge-retain binary.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    let config = Config::load(&cli.config)?;

    if cli.once {
        run_once(&config, cli.dry_run).await?;
        return Ok(());
    }

    let shutdown = AtomicBool::new(false);
    while !shutdown.load(Ordering::Relaxed) {
        let _ = run_once(&config, cli.dry_run).await;
        tokio::select! {
            () = sleep(config.poll_interval) => {}
            () = shutdown_signal() => {
                info!("Shutdown signal received, finishing current cycle...");
                shutdown.store(true, Ordering::Relaxed);
            }
        }
    }

    Ok(())
}

/// Run a single retention cycle across every configured host.
///
/// Hosts are processed sequentially in config order. A failure on one host
/// (password resolution, login, free-space query, torrent fetch) is logged
/// and the cycle continues with the next host — a single host failure does
/// not abort the whole cycle.
pub(crate) async fn run_once(config: &Config, dry_run: bool) -> Result<()> {
    for host in &config.hosts {
        process_host(host, &config.rules, dry_run).await;
    }
    info!("Cycle complete: processed {} hosts", config.hosts.len());
    Ok(())
}

/// Process a single host: resolve password, log in, check free space, and
/// execute a deletion plan if below the low water mark.
#[expect(
    clippy::too_many_lines,
    reason = "linear host-processing pipeline; splitting would obscure the sequential flow"
)]
async fn process_host(host: &HostConfig, rules: &config::Rules, dry_run: bool) {
    let password = match host.resolve_password() {
        Ok(pw) => pw,
        Err(err) => {
            error!(host = %host.host, port = host.port, error = %err, "failed to resolve password for host `{}:{}`: {err}", host.host, host.port);
            return;
        }
    };

    let client = DelugeClient::new(
        host.host.clone(),
        host.port,
        host.username.clone(),
        password,
    );

    if let Err(err) = client.login().await {
        error!(host = %host.host, port = host.port, error = %err, "login failed for host `{}:{}`: {err}", host.host, host.port);
        return;
    }

    let free_space = match client.get_free_space().await {
        Ok(bytes) => bytes,
        Err(err) => {
            error!(host = %host.host, port = host.port, error = %err, "free space query failed for host `{}:{}`: {err}", host.host, host.port);
            return;
        }
    };

    info!(
        host = %host.host,
        port = host.port,
        free = %ByteSize(free_space),
        low = %rules.low_water_mark,
        high = %rules.high_water_mark,
        "host {}:{}: free space {} (low: {}, high: {})",
        host.host,
        host.port,
        ByteSize(free_space),
        rules.low_water_mark,
        rules.high_water_mark,
    );

    if free_space >= rules.low_water_mark.as_u64() {
        info!(host = %host.host, port = host.port, "host {}:{}: OK", host.host, host.port);
        return;
    }

    warn!(host = %host.host, port = host.port, "host {}:{}: below low water mark, calculating deletion plan", host.host, host.port);

    let torrents = match client.get_torrents().await {
        Ok(list) => list,
        Err(err) => {
            error!(host = %host.host, port = host.port, error = %err, "torrent list query failed for host `{}:{}`: {err}", host.host, host.port);
            return;
        }
    };

    let now = Utc::now();
    let plan = compute_deletion_plan(
        &torrents,
        rules.min_seeders,
        rules.min_age_days,
        free_space,
        rules.high_water_mark.as_u64(),
        now,
    );

    if plan.is_empty() {
        warn!(host = %host.host, port = host.port, "host {}:{}: no eligible torrents", host.host, host.port);
        return;
    }

    let total_freed: u64 = plan.iter().map(|t| t.total_done).sum();
    info!(
        host = %host.host,
        port = host.port,
        count = plan.len(),
        freed = %ByteSize(total_freed),
        dry_run,
        "Plan: delete {} torrents, freeing ~{}{}",
        plan.len(),
        ByteSize(total_freed),
        if dry_run { " (DRY RUN)" } else { "" },
    );

    let throttle = Duration::from_secs(rules.delete_throttle_secs);
    if let Err(err) = execute_deletion_plan(&client, &plan, throttle, dry_run).await {
        error!(host = %host.host, port = host.port, error = %err, "deletion plan execution failed for host `{}:{}`: {err}", host.host, host.port);
        return;
    }

    if !dry_run {
        match client.get_free_space().await {
            Ok(new_free) => info!(
                host = %host.host,
                port = host.port,
                free = %ByteSize(new_free),
                "host {}:{}: free space after deletion: {}",
                host.host,
                host.port,
                ByteSize(new_free),
            ),
            Err(err) => error!(
                host = %host.host,
                port = host.port,
                error = %err,
                "post-deletion free space recheck failed for host `{}:{}`: {err}",
                host.host,
                host.port,
            ),
        }
    }
}

/// Wait for either `SIGINT` (Ctrl-C) or `SIGTERM` and return when either fires.
async fn shutdown_signal() {
    let mut sigterm = match unix::signal(SignalKind::terminate()) {
        Ok(stream) => stream,
        Err(err) => {
            error!(error = %err, "failed to install SIGTERM handler: {err}");
            // Fall back to waiting only for Ctrl-C; the select below still
            // completes when SIGINT arrives.
            ctrl_c().await.ok();
            return;
        }
    };

    tokio::select! {
        res = ctrl_c() => {
            if let Err(err) = res {
                error!(error = %err, "ctrl_c signal handler failed: {err}");
            }
        }
        opt = sigterm.recv() => {
            if opt.is_none() {
                error!("SIGTERM stream closed before signal arrived");
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "integration tests panic on unexpected shapes via expect for clarity"
)]
mod tests {
    // TODO(task-8): these integration tests use wiremock HTTP mocks for
    // the old Web UI JSON-RPC client. They are ignored until task 8
    // rewrites them against a daemon RPC mock server.
    use super::*;
    use serde_json::{Value, json};
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::config::Rules;

    const GB: u64 = 1_073_741_824;

    fn rules(low: u64, high: u64) -> Rules {
        Rules {
            min_seeders: 1,
            min_age_days: 1,
            low_water_mark: ByteSize(low),
            high_water_mark: ByteSize(high),
            delete_throttle_secs: 0,
        }
    }

    fn host(host_addr: String, port: u16) -> HostConfig {
        HostConfig {
            host: host_addr,
            port,
            username: String::from("localclient"),
            password: Some(String::from("secret")),
            password_env: None,
        }
    }

    fn config_with(server_uri: &str, rules: Rules) -> Config {
        // server_uri looks like "http://127.0.0.1:12345"
        let no_scheme = server_uri.strip_prefix("http://").unwrap_or(server_uri);
        let (host_addr, port_str) = no_scheme
            .split_once(':')
            .expect("mock server uri has host:port");
        let port: u16 = port_str.parse().expect("mock server port is u16");
        Config {
            poll_interval: Duration::from_secs(60),
            hosts: vec![host(host_addr.to_owned(), port)],
            rules,
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

    /// Mount a mock for `auth.login` returning success.
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

    /// A timestamp far enough in the past to satisfy `min_age_days = 1`.
    fn old_timestamp() -> f64 {
        let now = Utc::now().timestamp();
        i64_to_f64(now - 60 * 60 * 24 * 30)
    }

    #[expect(
        clippy::as_conversions,
        reason = "i64 to f64 for the Deluge time_added field is intentional"
    )]
    #[expect(
        clippy::cast_precision_loss,
        reason = "epoch seconds fit exactly in f64 mantissa for realistic timestamps"
    )]
    fn i64_to_f64(secs: i64) -> f64 {
        secs as f64
    }

    #[tokio::test]
    #[ignore = "task-8: rewrite against daemon RPC mock"]
    async fn when_dry_run_then_logs_plan_and_makes_no_remove_calls_should_skip_deletion() {
        let server = MockServer::start().await;
        mount_login(&server).await;
        // free space below low water mark (10 GB low, 20 GB high)
        mount_free_space(&server, 5 * GB).await;
        let torrents = json!({
            "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111": torrent_entry(
                "old-torrent",
                3.0,
                2 * GB,
                old_timestamp(),
            ),
        });
        mount_torrents(&server, torrents).await;

        let config = config_with(&server.uri(), rules(10 * GB, 20 * GB));

        let result = run_once(&config, true).await;

        assert!(result.is_ok());

        let received = server.received_requests().await.expect("read requests");
        let remove_calls = received
            .iter()
            .filter(|r| {
                let body: Value = serde_json::from_slice(&r.body).expect("body is json");
                body.get("method").and_then(|m| m.as_str()) == Some("core.remove_torrent")
            })
            .count();
        assert_eq!(
            remove_calls, 0,
            "dry run must not issue any core.remove_torrent calls"
        );
    }

    #[tokio::test]
    #[ignore = "task-8: rewrite against daemon RPC mock"]
    async fn when_not_dry_run_then_calls_remove_torrent_for_planned_torrents_should_delete() {
        let server = MockServer::start().await;
        mount_login(&server).await;
        mount_free_space(&server, 5 * GB).await;
        let info_hash = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111";
        let torrents = json!({
            info_hash: torrent_entry("old-torrent", 3.0, 2 * GB, old_timestamp()),
        });
        mount_torrents(&server, torrents).await;
        mount_remove(&server, info_hash).await;
        // Second free-space call (post-deletion recheck) — same mock matches.

        let config = config_with(&server.uri(), rules(10 * GB, 20 * GB));

        let result = run_once(&config, false).await;

        assert!(result.is_ok());

        let received = server.received_requests().await.expect("read requests");
        let remove_calls = received
            .iter()
            .filter(|r| {
                let body: Value = serde_json::from_slice(&r.body).expect("body is json");
                body.get("method").and_then(|m| m.as_str()) == Some("core.remove_torrent")
            })
            .count();
        assert_eq!(
            remove_calls, 1,
            "exactly one core.remove_torrent call expected for the planned torrent"
        );
    }

    #[tokio::test]
    #[ignore = "task-8: rewrite against daemon RPC mock"]
    async fn when_free_space_above_low_water_mark_then_no_plan_should_be_computed() {
        let server = MockServer::start().await;
        mount_login(&server).await;
        // free space above low water mark
        mount_free_space(&server, 30 * GB).await;

        let config = config_with(&server.uri(), rules(10 * GB, 20 * GB));

        let result = run_once(&config, false).await;

        assert!(result.is_ok());

        let received = server.received_requests().await.expect("read requests");
        let update_ui_calls = received
            .iter()
            .filter(|r| {
                let body: Value = serde_json::from_slice(&r.body).expect("body is json");
                body.get("method").and_then(|m| m.as_str()) == Some("web.update_ui")
            })
            .count();
        assert_eq!(
            update_ui_calls, 0,
            "no torrent list query when free space is above the low water mark"
        );
    }

    #[tokio::test]
    #[ignore = "task-8: rewrite against daemon RPC mock"]
    async fn when_login_fails_then_skips_host_and_cycle_succeeds_should_log_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/json"))
            .and(body_partial_json(json!({
                "method": "auth.login",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": false,
                "error": null,
                "id": 1,
            })))
            .mount(&server)
            .await;

        let config = config_with(&server.uri(), rules(10 * GB, 20 * GB));

        let result = run_once(&config, false).await;

        assert!(
            result.is_ok(),
            "cycle must succeed even when a host fails to log in"
        );

        let received = server.received_requests().await.expect("read requests");
        let post_login_calls = received
            .iter()
            .filter(|r| {
                let body: Value = serde_json::from_slice(&r.body).expect("body is json");
                let method = body.get("method").and_then(|m| m.as_str());
                method != Some("auth.login")
            })
            .count();
        assert_eq!(
            post_login_calls, 0,
            "no further API calls after a login failure"
        );
    }

    #[tokio::test]
    #[ignore = "task-8: rewrite against daemon RPC mock"]
    async fn when_no_eligible_torrents_then_plan_is_empty_should_log_no_eligible() {
        let server = MockServer::start().await;
        mount_login(&server).await;
        mount_free_space(&server, 5 * GB).await;
        // A young torrent (added now) — filtered out by min_age_days = 1.
        let now_secs = Utc::now().timestamp();
        let young = i64_to_f64(now_secs);
        let torrents = json!({
            "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222": torrent_entry(
                "young-torrent",
                3.0,
                2 * GB,
                young,
            ),
        });
        mount_torrents(&server, torrents).await;

        let config = config_with(&server.uri(), rules(10 * GB, 20 * GB));

        let result = run_once(&config, true).await;

        assert!(result.is_ok());

        let received = server.received_requests().await.expect("read requests");
        let remove_calls = received
            .iter()
            .filter(|r| {
                let body: Value = serde_json::from_slice(&r.body).expect("body is json");
                body.get("method").and_then(|m| m.as_str()) == Some("core.remove_torrent")
            })
            .count();
        assert_eq!(
            remove_calls, 0,
            "no deletions when no torrents are eligible"
        );
    }
}
