//! Retention engine - decides which torrents to remove and executes the plan.
//!
//! The engine is a pure planner ([`compute_deletion_plan`]) plus a thin
//! executor ([`execute_deletion_plan`]) that talks to the Deluge daemon.
//! Planning is deterministic and side-effect free so it can be unit-tested
//! without a live client; execution is async and throttled.

use crate::policy::filter_eligible;
use anyhow::Result;
use bytesize::ByteSize;
use chrono::{DateTime, Utc};
use deluge_rpc::CoreTorrentRpc;
use deluge_rpc::models::torrents::TorrentEntry;
use std::cmp::Ordering;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Compute the ordered list of torrents to delete in order to bring
/// `free_space` up to `high_water_mark`.
///
/// The algorithm is:
///
/// 1. Filter `torrents` down to those eligible for deletion via
///    [`filter_eligible`] (completed, healthy swarm, old enough).
/// 2. Sort the eligible torrents by `ratio` **descending** - the torrent
///    that has already given the most back to the swarm is deleted first.
/// 3. Greedily walk the sorted list, accumulating `total_done` (bytes on
///    disk). Stop as soon as `free_space + accumulated >= high_water_mark`.
/// 4. Return the selected torrents in deletion order (highest ratio first).
///
/// When no eligible torrents exist, an empty plan is returned and the
/// caller is responsible for warning that free space is still below the
/// water mark. When every eligible torrent is selected but the
/// accumulated bytes still do not reach `high_water_mark`, the full
/// eligible list is returned and the caller warns that deletion was
/// insufficient.
pub fn compute_deletion_plan(
    torrents: &[TorrentEntry],
    min_seeders: u32,
    min_age_days: u64,
    free_space: u64,
    high_water_mark: u64,
    now: DateTime<Utc>,
) -> Vec<TorrentEntry> {
    let eligible = filter_eligible(torrents, min_seeders, min_age_days, now);
    if eligible.is_empty() {
        return Vec::new();
    }

    let needed = high_water_mark.saturating_sub(free_space);
    if needed == 0 {
        return Vec::new();
    }

    let mut sorted: Vec<TorrentEntry> = eligible.into_iter().cloned().collect();
    sorted.sort_by(|a, b| {
        b.status
            .ratio
            .unwrap_or(-1.0)
            .partial_cmp(&a.status.ratio.unwrap_or(-1.0))
            .unwrap_or(Ordering::Equal)
    });

    let mut accumulated: u64 = 0;

    let mut plan: Vec<TorrentEntry> = Vec::new();
    for torrent in sorted {
        let size = u64::try_from(torrent.status.total_done).unwrap_or(0);
        plan.push(torrent);
        accumulated = accumulated.saturating_add(size);
        if accumulated >= needed {
            break;
        }
    }

    if accumulated < needed {
        warn!(
            eligible_count = plan.len(),
            "deletion plan frees less than high_water_mark; all eligible torrents selected"
        );
    }

    plan
}

/// Execute a deletion plan against a Deluge daemon.
///
/// When `dry_run` is `true`, each torrent that *would* be deleted is logged
/// via `tracing::info!` (name, ratio, on-disk size) and no API calls are
/// made - this is the safe preview path.
///
/// When `dry_run` is `false`, each torrent is removed via
/// [`CoreTorrentRpc::remove_torrent`]. A failure on one torrent is logged via
/// `tracing::error!` and the loop continues with the next torrent - a single
/// failure does not abort the whole plan. Between deletions (except after
/// the last one) the coroutine sleeps for `throttle` to avoid hammering the
/// daemon.
///
/// # Errors
///
/// Returns `Err` only when the underlying client call returns an error that
/// cannot be recovered per-torrent; per-torrent removal failures are logged
/// and swallowed so the plan completes.
pub async fn execute_deletion_plan(
    client: &dyn CoreTorrentRpc,
    plan: &[TorrentEntry],
    throttle: Duration,
    dry_run: bool,
) -> Result<()> {
    if plan.is_empty() {
        info!("no torrents to delete - plan is empty");
        return Ok(());
    }

    if dry_run {
        info!(
            count = plan.len(),
            "dry run: would delete {} torrent(s)",
            plan.len()
        );
        for torrent in plan {
            info!(
                name = %torrent.status.name,
                ratio = torrent.status.ratio.unwrap_or(-1.0),
                size = %ByteSize(u64::try_from(torrent.status.total_done).unwrap_or(0)),
                "would delete torrent `{}` (ratio {}, size {})",
                torrent.status.name,
                torrent.status.ratio.unwrap_or(-1.0),
                ByteSize(u64::try_from(torrent.status.total_done).unwrap_or(0)),
            );
        }
        return Ok(());
    }

    let last_idx = plan.len().saturating_sub(1);
    for (idx, torrent) in plan.iter().enumerate() {
        match client.remove_torrent(&torrent.info_hash, true).await {
            Ok(true) => info!(
                name = %torrent.status.name,
                info_hash = %torrent.info_hash,
                "deleted torrent `{}`",
                torrent.status.name,
            ),
            Ok(false) => warn!(
                name = %torrent.status.name,
                info_hash = %torrent.info_hash,
                "daemon reported torrent `{}` not removed (already gone?)",
                torrent.status.name,
            ),
            Err(err) => error!(
                name = %torrent.status.name,
                info_hash = %torrent.info_hash,
                error = %err,
                "failed to remove torrent `{}`: {err}",
                torrent.status.name,
            ),
        }

        if idx < last_idx && !throttle.is_zero() {
            sleep(throttle).await;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use deluge_rpc::MockCoreTorrentRpc;
    use deluge_rpc::models::torrents::TorrentStatus;

    const GB: i64 = 1_073_741_824;

    fn make_torrent(
        info_hash: &str,
        name: &str,
        ratio: Option<f64>,
        total_done: i64,
    ) -> TorrentEntry {
        TorrentEntry {
            info_hash: String::from(info_hash),
            status: TorrentStatus {
                name: String::from(name),
                state: String::from("Seeding"),
                progress: 100.0,
                ratio,
                total_seeds: 50,
                num_seeds: 5,
                time_added: 1_700_000_000,
                total_done,
                total_uploaded: 0,
                is_finished: true,
                download_location: String::from("/data"),
                ..Default::default()
            },
        }
    }

    fn now() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(1_700_000_000 + 60 * 60 * 24 * 30, 0)
            .expect("fixed test timestamp parses")
    }

    #[test]
    fn when_canonical_fixture_then_plan_stops_at_two_torrents_should_match_spec() {
        let torrents = vec![
            make_torrent("aaa", "big-low-ratio", Some(1.0), 5 * GB),
            make_torrent("bbb", "mid-ratio", Some(2.0), 2 * GB),
            make_torrent("ccc", "small-high-ratio", Some(3.0), GB),
        ];
        let free_space = 10 * GB as u64;
        let high_water_mark = 13 * GB as u64;

        let plan = compute_deletion_plan(&torrents, 1, 1, free_space, high_water_mark, now());

        assert_eq!(plan.len(), 2, "should stop once 3GB freed");
        assert_eq!(plan[0].info_hash, "ccc", "highest ratio first");
        assert_eq!(plan[1].info_hash, "bbb", "second highest ratio second");
    }

    #[test]
    fn when_ratios_unsorted_then_deletion_order_is_descending_should_sort_by_ratio() {
        let torrents = vec![
            make_torrent("low", "low", Some(0.5), GB),
            make_torrent("high", "high", Some(2.0), GB),
            make_torrent("mid", "mid", Some(1.0), GB),
        ];

        let plan = compute_deletion_plan(&torrents, 1, 1, 0, u64::MAX, now());

        assert_eq!(
            plan.len(),
            3,
            "all selected when high_water_mark unreachable"
        );
        assert_eq!(plan[0].status.ratio, Some(2.0));
        assert_eq!(plan[1].status.ratio, Some(1.0));
        assert_eq!(plan[2].status.ratio, Some(0.5));
    }

    #[test]
    fn when_single_oversized_torrent_then_still_selected_should_include_oversized() {
        let torrents = vec![make_torrent("big", "big", Some(1.0), 5 * GB)];

        let plan = compute_deletion_plan(
            &torrents,
            1,
            1,
            10 * GB as u64,
            13 * GB as u64,
            now(),
        );

        assert_eq!(plan.len(), 1, "oversized torrent is still selected");
        assert_eq!(plan[0].info_hash, "big");
    }

    #[test]
    fn when_no_eligible_torrents_then_empty_plan_should_be_returned() {
        let torrents = vec![make_torrent("young", "young", Some(1.0), GB)];

        let plan = compute_deletion_plan(&torrents, 1, 365, 0, 1, now());

        assert!(plan.is_empty());
    }

    #[test]
    fn when_all_eligible_selected_but_still_below_high_water_mark_then_returns_all_should_warn() {
        let torrents = vec![
            make_torrent("a", "a", Some(1.0), GB),
            make_torrent("b", "b", Some(2.0), GB),
        ];

        let plan = compute_deletion_plan(&torrents, 1, 1, 0, 100 * GB as u64, now());

        assert_eq!(
            plan.len(),
            2,
            "all eligible returned even if insufficient"
        );
    }

    #[test]
    fn when_empty_input_then_empty_plan_should_be_returned() {
        let plan = compute_deletion_plan(&Vec::new(), 1, 1, 0, 1, now());
        assert!(plan.is_empty());
    }

    #[test]
    fn when_free_space_already_above_high_water_mark_then_empty_plan_should_be_returned() {
        let torrents = vec![make_torrent("a", "a", Some(1.0), GB)];

        let plan = compute_deletion_plan(
            &torrents,
            1,
            1,
            20 * GB as u64,
            10 * GB as u64,
            now(),
        );

        assert!(
            plan.is_empty(),
            "no deletion needed when free_space >= high_water_mark"
        );
    }

    #[tokio::test]
    async fn when_dry_run_then_no_api_calls_and_returns_ok_should_log_only() {
        let mut client = MockCoreTorrentRpc::new();
        client.expect_remove_torrent().never();

        let plan = vec![
            make_torrent("aaa", "first", Some(3.0), GB),
            make_torrent("bbb", "second", Some(2.0), 2 * GB),
        ];

        let result =
            execute_deletion_plan(&client, &plan, Duration::from_millis(0), true).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn when_empty_plan_then_returns_ok_and_makes_no_calls_should_short_circuit() {
        let mut client = MockCoreTorrentRpc::new();
        client.expect_remove_torrent().never();

        let result =
            execute_deletion_plan(&client, &[], Duration::from_millis(0), false).await;

        assert!(result.is_ok());
    }

    #[test]
    fn when_torrents_have_equal_ratios_then_sort_is_stable_should_preserve_input_order() {
        let torrents = vec![
            make_torrent("first", "first", Some(1.0), GB),
            make_torrent("second", "second", Some(1.0), GB),
            make_torrent("third", "third", Some(1.0), GB),
        ];

        let plan = compute_deletion_plan(&torrents, 1, 1, 0, 1, now());

        assert_eq!(
            plan.len(),
            1,
            "only one torrent needed to reach high_water_mark"
        );
        assert_eq!(
            plan[0].info_hash, "first",
            "stable sort preserves input order on ties"
        );
    }

    #[test]
    fn when_exact_fit_then_plan_stops_inclusively_should_match_boundary() {
        let torrents = vec![
            make_torrent("a", "a", Some(3.0), GB),
            make_torrent("b", "b", Some(2.0), 2 * GB),
            make_torrent("c", "c", Some(1.0), 5 * GB),
        ];

        let plan = compute_deletion_plan(
            &torrents,
            1,
            1,
            10 * GB as u64,
            13 * GB as u64,
            now(),
        );

        assert_eq!(
            plan.len(),
            2,
            "boundary: accumulated == needed stops the loop"
        );
        assert_eq!(plan[0].info_hash, "a");
        assert_eq!(plan[1].info_hash, "b");
    }

    #[test]
    fn when_ratio_is_negative_one_then_sorts_last_should_be_lowest_priority() {
        let torrents = vec![
            make_torrent("normal", "normal", Some(1.0), GB),
            make_torrent("infinite", "infinite", None, GB),
            make_torrent("high", "high", Some(3.0), GB),
        ];
        let plan = compute_deletion_plan(&torrents, 1, 1, 0, (2 * GB) as u64, now());
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].info_hash, "high");
        assert_eq!(plan[1].info_hash, "normal");
    }
}
