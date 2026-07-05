//! Entry point for the deluge-retain binary.
//!
//! Wires together the CLI parser, tracing init, config loader, and the
//! retention engine into the two operating modes:
//!
//! - **once mode** (`--once`): run a single retention cycle and exit.
//! - **watch mode** (default): poll every host on `config.poll_interval`,
//!   handling `SIGINT`/`SIGTERM` for graceful shutdown between cycles.

use anyhow::Result;
use bytesize::ByteSize;
use chrono::Utc;
use clap::Parser;
use deluge_retain::cli::Cli;
use deluge_retain::config::{Config, HostConfig, Rules};
use deluge_retain::engine::{compute_deletion_plan, execute_deletion_plan};
use deluge_retain::tracing_setup::init_tracing;
use deluge_rpc::CoreSessionRpc;
use deluge_rpc::CoreTorrentRpc;
use deluge_rpc::DelugeClient;
use deluge_rpc::models::torrents::FilterDict;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::signal::unix::SignalKind;
use tokio::signal::{ctrl_c, unix};
use tokio::time::sleep;
use tracing::{error, info, warn};

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
/// and the cycle continues with the next host - a single host failure does
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
async fn process_host(host: &HostConfig, rules: &Rules, dry_run: bool) {
    let password = match host.resolve_password() {
        Ok(pw) => pw,
        Err(err) => {
            error!(host = %host.host, port = host.port, error = %err, "failed to resolve password for host `{}:{}`: {err}", host.host, host.port);
            return;
        }
    };

    let client = match DelugeClient::connect(&host.host, host.port, &host.username, &password).await
    {
        Ok(client) => client,
        Err(err) => {
            error!(host = %host.host, port = host.port, error = %err, "connection/login failed for host `{}:{}`: {err}", host.host, host.port);
            return;
        }
    };

    let free_space = match client.core().session.get_free_space(None).await {
        Ok(bytes) => u64::try_from(bytes).unwrap_or(0),
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

    let keys = [
        "name",
        "state",
        "progress",
        "ratio",
        "total_seeds",
        "num_seeds",
        "time_added",
        "total_done",
        "total_uploaded",
        "is_finished",
        "download_location",
    ]
    .map(String::from)
    .to_vec();

    let torrents = match client
        .core()
        .torrents
        .get_torrents_status(&FilterDict::default(), &keys, false)
        .await
    {
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

    let total_freed: u64 = plan
        .iter()
        .map(|t| u64::try_from(t.status.total_done).unwrap_or(0))
        .sum();
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
    if let Err(err) = execute_deletion_plan(&client.core().torrents, &plan, throttle, dry_run).await
    {
        error!(host = %host.host, port = host.port, error = %err, "deletion plan execution failed for host `{}:{}`: {err}", host.host, host.port);
        return;
    }

    if !dry_run {
        match client.core().session.get_free_space(None).await {
            Ok(new_free) => info!(
                host = %host.host,
                port = host.port,
                free = %ByteSize(u64::try_from(new_free).unwrap_or(0)),
                "host {}:{}: free space after deletion: {}",
                host.host,
                host.port,
                ByteSize(u64::try_from(new_free).unwrap_or(0)),
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
#[path = "../tests/common/cassettes.rs"]
mod cassettes;

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rpc_mock::Matcher;
    use deluge_rpc_mock::ReplayServer;

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

    fn config_with(server: &ReplayServer, rules: Rules) -> Config {
        Config {
            poll_interval: Duration::from_secs(60),
            hosts: vec![host(server.host(), server.port())],
            rules,
        }
    }

    fn old_timestamp() -> i64 {
        Utc::now().timestamp() - 60 * 60 * 24 * 30
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_dry_run_then_logs_plan_and_makes_no_remove_calls() {
        let cassette = cassettes::torrents_list(
            "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111",
            "old-torrent",
            old_timestamp(),
        );
        let matcher = Matcher::new(cassette.interactions);
        let server = ReplayServer::start(matcher)
            .await
            .expect("start replay server");
        let config = config_with(&server, rules(10 * GB, 20 * GB));

        let result = run_once(&config, true).await;

        assert!(result.is_ok());

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

    #[tokio::test(flavor = "multi_thread")]
    async fn when_not_dry_run_then_calls_remove_torrent() {
        let info_hash = "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111";
        let cassette = cassettes::remove_torrent(info_hash, old_timestamp());
        let matcher = Matcher::new(cassette.interactions);
        let server = ReplayServer::start(matcher)
            .await
            .expect("start replay server");
        let config = config_with(&server, rules(10 * GB, 20 * GB));

        let result = run_once(&config, false).await;

        assert!(result.is_ok());

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

    #[tokio::test(flavor = "multi_thread")]
    async fn when_free_space_above_low_water_mark_then_no_torrent_query() {
        let cassette = cassettes::free_space_high();
        let matcher = Matcher::new(cassette.interactions);
        let server = ReplayServer::start(matcher)
            .await
            .expect("start replay server");
        let config = config_with(&server, rules(10 * GB, 20 * GB));

        let result = run_once(&config, false).await;

        assert!(result.is_ok());

        let methods = server.consumed_methods();
        assert!(
            !methods.iter().any(|m| m == "core.get_torrents_status"),
            "no torrent list query when free space is above the low water mark, got {methods:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn when_no_eligible_torrents_then_plan_is_empty() {
        let now_secs = Utc::now().timestamp();
        let cassette = cassettes::torrents_list(
            "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222",
            "young-torrent",
            now_secs,
        );
        let matcher = Matcher::new(cassette.interactions);
        let server = ReplayServer::start(matcher)
            .await
            .expect("start replay server");
        let config = config_with(&server, rules(10 * GB, 20 * GB));

        let result = run_once(&config, true).await;

        assert!(result.is_ok());

        let methods = server.consumed_methods();
        let remove_calls = methods
            .iter()
            .filter(|m| *m == "core.remove_torrent")
            .count();
        assert_eq!(
            remove_calls, 0,
            "no deletions when no torrents are eligible, got {methods:?}"
        );
    }
}
