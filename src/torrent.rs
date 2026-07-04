//! Torrent data model and state representation.

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Status snapshot for a single torrent, as returned by Deluge's
/// `web.update_ui` JSON-RPC method.
///
/// `info_hash` is not part of the per-torrent status dict — it is the
/// key under which the dict is nested — so it is skipped during
/// deserialization and populated by the client after parsing.
///
/// # Field semantics
///
/// `total_seeds` is the swarm-wide seed count reported by the tracker
/// (the size of the seeder pool for this torrent's infohash). `num_seeds`
/// is the number of seeds the local Deluge daemon is currently connected
/// to. The retention filter cares about swarm health, so it uses
/// `total_seeds` — a torrent with a healthy seeder pool elsewhere can be
/// safely deleted locally even if the daemon happens to be connected to
/// few peers right now.
#[derive(Debug, Clone, Deserialize)]
pub struct TorrentInfo {
    #[serde(skip)]
    pub info_hash: String,
    pub name: String,
    #[expect(
        dead_code,
        reason = "populated by the client; not yet read by the engine or main"
    )]
    pub state: String,
    pub progress: f64,
    /// Deluge returns -1.0 when `total_done` == 0 (ratio is infinite). The
    /// descending-ratio sort in `compute_deletion_plan` naturally places
    /// these last (lowest deletion priority).
    pub ratio: f64,
    pub total_seeds: u32,
    #[expect(
        dead_code,
        reason = "populated by the client; not yet read by the engine or main"
    )]
    pub num_seeds: u32,
    pub time_added: i64,
    pub total_done: u64,
    #[expect(
        dead_code,
        reason = "populated by the client; not yet read by the engine or main"
    )]
    pub total_uploaded: u64,
    pub is_finished: bool,
    #[expect(
        dead_code,
        reason = "populated by the client; not yet read by the engine or main"
    )]
    pub download_location: String,
}

/// Convert a Deluge `time_added` epoch-seconds value into a UTC `DateTime`.
///
/// Returns `None` when the value is out of range, so callers can fall
/// back to a sensible default (typically treating the torrent as
/// ineligible rather than crashing).
fn time_added_to_datetime(time_added: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(time_added, 0)
}

/// Filter `torrents` down to those eligible for deletion under the
/// retention rules.
///
/// A torrent is eligible when **all** of the following hold:
///
/// 1. **Completed** — `is_finished` is `true` *or* `progress >= 100.0`.
///    This is hardcoded; there is no config flag to retain incomplete
///    torrents.
/// 2. **Healthy swarm** — `total_seeds >= min_seeders`. The swarm-wide
///    seeder count is used (see [`TorrentInfo`] field docs), not the
///    count of locally connected seeds.
/// 3. **Old enough** — the torrent was added at least `min_age_days`
///    days before `now`. Torrents whose `time_added` cannot be parsed
///    are treated as age-zero (newest) and therefore filtered out.
///
/// Returns references to the eligible torrents in input order.
pub fn filter_eligible(
    torrents: &[TorrentInfo],
    min_seeders: u32,
    min_age_days: u64,
    now: DateTime<Utc>,
) -> Vec<&TorrentInfo> {
    let min_age_days_i = i64::try_from(min_age_days).unwrap_or(i64::MAX);

    torrents
        .iter()
        .filter(|t| {
            let completed = t.is_finished || t.progress >= 100.0;
            if !completed {
                return false;
            }
            if t.total_seeds < min_seeders {
                return false;
            }
            match time_added_to_datetime(t.time_added) {
                Some(added) => (now - added).num_days() >= min_age_days_i,
                None => false,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_torrent(
        progress: f64,
        total_seeds: u32,
        time_added: DateTime<Utc>,
        is_finished: bool,
    ) -> TorrentInfo {
        TorrentInfo {
            info_hash: String::from("deadbeef"),
            name: String::from("test"),
            state: String::from("Seeding"),
            progress,
            ratio: 1.0,
            total_seeds,
            num_seeds: 0,
            time_added: time_added.timestamp(),
            total_done: 0,
            total_uploaded: 0,
            is_finished,
            download_location: String::from("/data"),
        }
    }

    #[test]
    fn when_completed_with_enough_seeders_and_age_then_eligible_should_be_in_result() {
        let now = Utc::now();
        let added = now - Duration::days(30);
        let t = make_torrent(100.0, 15, added, false);
        let torrents = [t.clone()];

        let result = filter_eligible(&torrents, 10, 28, now);

        assert_eq!(result.len(), 1);
        if let Some(eligible) = result.first() {
            assert_eq!(eligible.info_hash, t.info_hash);
        }
    }

    #[test]
    fn when_not_completed_then_filtered_out_should_be_excluded() {
        let now = Utc::now();
        let added = now - Duration::days(30);
        let t = make_torrent(50.0, 15, added, false);
        let torrents = [t];

        let result = filter_eligible(&torrents, 10, 28, now);

        assert!(result.is_empty());
    }

    #[test]
    fn when_not_enough_seeders_then_filtered_out_should_be_excluded() {
        let now = Utc::now();
        let added = now - Duration::days(30);
        let t = make_torrent(100.0, 5, added, false);
        let torrents = [t];

        let result = filter_eligible(&torrents, 10, 28, now);

        assert!(result.is_empty());
    }

    #[test]
    fn when_too_young_then_filtered_out_should_be_excluded() {
        let now = Utc::now();
        let added = now - Duration::days(10);
        let t = make_torrent(100.0, 15, added, false);
        let torrents = [t];

        let result = filter_eligible(&torrents, 10, 28, now);

        assert!(result.is_empty());
    }

    #[test]
    fn when_progress_just_below_100_then_filtered_out_should_be_excluded() {
        let now = Utc::now();
        let added = now - Duration::days(30);
        let t = make_torrent(99.9, 15, added, false);
        let torrents = [t];

        let result = filter_eligible(&torrents, 10, 28, now);

        assert!(result.is_empty());
    }

    #[test]
    fn when_progress_100_but_not_finished_then_eligible_should_be_in_result() {
        let now = Utc::now();
        let added = now - Duration::days(30);
        let t = make_torrent(100.0, 15, added, false);
        let torrents = [t];

        let result = filter_eligible(&torrents, 10, 28, now);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn when_finished_but_progress_low_then_eligible_should_be_in_result() {
        let now = Utc::now();
        let added = now - Duration::days(30);
        let t = make_torrent(50.0, 15, added, true);
        let torrents = [t];

        let result = filter_eligible(&torrents, 10, 28, now);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn when_total_seeds_exactly_at_threshold_then_eligible_should_be_in_result() {
        let now = Utc::now();
        let added = now - Duration::days(30);
        let t = make_torrent(100.0, 10, added, false);
        let torrents = [t];

        let result = filter_eligible(&torrents, 10, 28, now);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn when_age_exactly_at_threshold_then_eligible_should_be_in_result() {
        let now = Utc::now();
        let added = now - Duration::days(28);
        let t = make_torrent(100.0, 15, added, false);
        let torrents = [t];

        let result = filter_eligible(&torrents, 10, 28, now);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn when_empty_input_then_empty_result_should_be_returned() {
        let now = Utc::now();
        let result = filter_eligible(&[], 10, 28, now);

        assert!(result.is_empty());
    }

    #[test]
    fn when_time_added_out_of_range_then_filtered_out_should_be_excluded() {
        let now = Utc::now();
        let mut t = make_torrent(100.0, 15, now - Duration::days(30), false);
        t.time_added = i64::MIN;
        let torrents = [t];

        let result = filter_eligible(&torrents, 10, 28, now);

        assert!(result.is_empty());
    }
}
