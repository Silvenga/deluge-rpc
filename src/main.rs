//! Entry point for the deluge-retain binary.
//!
//! Wires together the CLI parser, tracing init, config loader, and the
//! retention engine into the two operating modes:
//!
//! - **once mode** (`--once`): run a single retention cycle and exit.
//! - **watch mode** (default): poll every host on `config.poll_interval`,
//!   handling `SIGINT`/`SIGTERM` for graceful shutdown between cycles.

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

use deluge_retain::cli::Cli;
use deluge_retain::client::{DelugeClient, DelugeRpc};
use deluge_retain::config::{Config, HostConfig, Rules};
use deluge_retain::engine::{compute_deletion_plan, execute_deletion_plan};
use deluge_retain::tracing_setup::init_tracing;

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
async fn process_host(host: &HostConfig, rules: &Rules, dry_run: bool) {
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
    let client_dyn: &dyn DelugeRpc = &client;
    if let Err(err) = execute_deletion_plan(client_dyn, &plan, throttle, dry_run).await {
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
#[path = "../tests/common/mock_daemon.rs"]
mod mock_daemon;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "integration tests use unwrap for clarity on known-good values"
)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use deluge_retain::rencode::RencodeValue;

    use mock_daemon::{MockDaemonConfig, MockDelugeDaemon, MockResponse};

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

    fn config_with(mock: &MockDelugeDaemon, rules: Rules) -> Config {
        Config {
            poll_interval: Duration::from_secs(60),
            hosts: vec![host(mock.host(), mock.port())],
            rules,
        }
    }

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

    #[tokio::test]
    async fn when_dry_run_then_logs_plan_and_makes_no_remove_calls() {
        let config_mock = MockDaemonConfig {
            login: MockResponse::success(RencodeValue::Int(5)),
            free_space: MockResponse::success(RencodeValue::Int(i64::try_from(5 * GB).unwrap())),
            torrents: MockResponse::success(torrents_dict(
                "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111",
                "old-torrent",
                3.0,
                2 * GB,
                old_timestamp(),
            )),
            remove: MockResponse::success(RencodeValue::Bool(true)),
        };
        let mock = MockDelugeDaemon::start(config_mock).await;
        let config = config_with(&mock, rules(10 * GB, 20 * GB));

        let result = run_once(&config, true).await;

        assert!(result.is_ok());

        let methods = mock.received_methods();
        let remove_calls = methods.iter().filter(|m| *m == "core.remove_torrent").count();
        assert_eq!(remove_calls, 0, "dry run must not issue any core.remove_torrent calls, got {methods:?}");
    }

    #[tokio::test]
    async fn when_not_dry_run_then_calls_remove_torrent() {
        let info_hash = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111";
        let config_mock = MockDaemonConfig {
            login: MockResponse::success(RencodeValue::Int(5)),
            free_space: MockResponse::success(RencodeValue::Int(i64::try_from(5 * GB).unwrap())),
            torrents: MockResponse::success(torrents_dict(info_hash, "old-torrent", 3.0, 2 * GB, old_timestamp())),
            remove: MockResponse::success(RencodeValue::Bool(true)),
        };
        let mock = MockDelugeDaemon::start(config_mock).await;
        let config = config_with(&mock, rules(10 * GB, 20 * GB));

        let result = run_once(&config, false).await;

        assert!(result.is_ok());

        let methods = mock.received_methods();
        let remove_calls = methods.iter().filter(|m| *m == "core.remove_torrent").count();
        assert_eq!(remove_calls, 1, "exactly one core.remove_torrent call expected, got {methods:?}");
    }

    #[tokio::test]
    async fn when_free_space_above_low_water_mark_then_no_torrent_query() {
        let config_mock = MockDaemonConfig {
            login: MockResponse::success(RencodeValue::Int(5)),
            free_space: MockResponse::success(RencodeValue::Int(i64::try_from(30 * GB).unwrap())),
            torrents: MockResponse::NotConfigured,
            remove: MockResponse::NotConfigured,
        };
        let mock = MockDelugeDaemon::start(config_mock).await;
        let config = config_with(&mock, rules(10 * GB, 20 * GB));

        let result = run_once(&config, false).await;

        assert!(result.is_ok());

        let methods = mock.received_methods();
        assert!(
            !methods.iter().any(|m| m == "core.get_torrents_status"),
            "no torrent list query when free space is above the low water mark, got {methods:?}"
        );
    }

    #[tokio::test]
    async fn when_login_fails_then_skips_host() {
        let mock = MockDelugeDaemon::start(MockDaemonConfig::login_bad()).await;
        let config = config_with(&mock, rules(10 * GB, 20 * GB));

        let result = run_once(&config, false).await;

        assert!(result.is_ok(), "cycle must succeed even when a host fails to log in");

        let methods = mock.received_methods();
        let post_login_calls = methods.iter().filter(|m| *m != "daemon.login").count();
        assert_eq!(post_login_calls, 0, "no further API calls after a login failure, got {methods:?}");
    }

    #[tokio::test]
    async fn when_no_eligible_torrents_then_plan_is_empty() {
        let now_secs = Utc::now().timestamp();
        let config_mock = MockDaemonConfig {
            login: MockResponse::success(RencodeValue::Int(5)),
            free_space: MockResponse::success(RencodeValue::Int(i64::try_from(5 * GB).unwrap())),
            torrents: MockResponse::success(torrents_dict(
                "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222",
                "young-torrent",
                3.0,
                2 * GB,
                now_secs,
            )),
            remove: MockResponse::success(RencodeValue::Bool(true)),
        };
        let mock = MockDelugeDaemon::start(config_mock).await;
        let config = config_with(&mock, rules(10 * GB, 20 * GB));

        let result = run_once(&config, true).await;

        assert!(result.is_ok());

        let methods = mock.received_methods();
        let remove_calls = methods.iter().filter(|m| *m == "core.remove_torrent").count();
        assert_eq!(remove_calls, 0, "no deletions when no torrents are eligible, got {methods:?}");
    }
}
