use bytesize::ByteSize;
use clap::Args;
use deluge_rpc_client::DelugeClient;
use deluge_rpc_client::RencodeValue;
use deluge_rpc_client::models::FilterDict;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};

/// High-level status overview.
#[derive(Args, Debug, Clone)]
pub struct StatusCommand;

impl StatusCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<String> {
        let session_keys: Vec<String> = [
            "download_rate",
            "upload_rate",
            "payload_download_rate",
            "payload_upload_rate",
            "ip_overhead_download_rate",
            "ip_overhead_upload_rate",
            "tracker_download_rate",
            "tracker_upload_rate",
            "dht_download_rate",
            "dht_upload_rate",
            "write_hit_ratio",
            "read_hit_ratio",
            "peer.num_peers_connected",
            "dht.dht_nodes",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let torrent_keys: Vec<String> = vec!["state".into(), "total_wanted".into()];
        let filter_dict = FilterDict::default();

        let (session, config, external_ip, free_space, entries) = tokio::try_join!(
            client.core.session.get_session_status(&session_keys),
            client.core.config.get_config(),
            client.core.session.get_external_ip(),
            client.core.session.get_free_space(None),
            client
                .core
                .torrents
                .get_torrents_status(&filter_dict, &torrent_keys, false),
        )?;

        let connections = extra_i64(&session.extra, "peer.num_peers_connected")?;
        let dht_nodes = extra_i64(&session.extra, "dht.dht_nodes")?;

        let download_payload = session.payload_download_rate.max(0.0);
        let upload_payload = session.payload_upload_rate.max(0.0);
        let download_overhead = (session.download_rate - session.payload_download_rate).max(0.0);
        let upload_overhead = (session.upload_rate - session.payload_upload_rate).max(0.0);

        let mut disk_usage: i64 = 0;
        let mut state_counts: BTreeMap<String, i64> = BTreeMap::new();
        for entry in &entries {
            *state_counts.entry(entry.status.state.clone()).or_insert(0) += 1;
            disk_usage = disk_usage.saturating_add(entry.status.total_wanted);
        }

        let mut out = String::new();

        let connections_line = match config.max_connections_global {
            Some(limit) => format!("{connections}/{limit}"),
            None => format!("{connections}"),
        };
        let dht_suffix = if dht_nodes > 0 {
            format!(" ({dht_nodes} DHT)")
        } else {
            String::new()
        };
        out.push_str(&format!("Connections:    {connections_line}{dht_suffix}\n"));

        out.push_str(&format!(
            "Download Speed: {}{} ({})\n",
            format_rate(download_payload),
            format_speed_limit(config.max_download_speed),
            format_rate(download_overhead),
        ));
        out.push_str(&format!(
            "Upload Speed:   {}{} ({})\n",
            format_rate(upload_payload),
            format_speed_limit(config.max_upload_speed),
            format_rate(upload_overhead),
        ));

        out.push_str(&format!(
            "Disk Free:      {}\n",
            ByteSize::b(free_space.max(0) as u64),
        ));
        out.push_str(&format!(
            "Disk Usage:     {}\n",
            ByteSize::b(disk_usage.max(0) as u64),
        ));

        let ip = if external_ip.is_empty() {
            "n/a".to_owned()
        } else {
            external_ip
        };
        out.push_str(&format!("External IP:    {ip}\n"));

        if !state_counts.is_empty() {
            out.push('\n');
            for state in order_states(state_counts.keys()) {
                if let Some(count) = state_counts.get(state) {
                    out.push_str(&format!("{state}: {count}\n"));
                }
            }
        }

        if out.ends_with('\n') {
            out.pop();
        }

        Ok(out)
    }
}

/// Extracts an `i64` from the session status `extra` map, defaulting to 0 when
/// the key is absent (e.g. DHT disabled returns no `dht.dht_nodes`).
fn extra_i64(extra: &HashMap<String, RencodeValue>, key: &str) -> anyhow::Result<i64> {
    match extra.get(key) {
        Some(v) => Ok(i64::deserialize(v)?),
        None => Ok(0),
    }
}

/// Formats a byte/sec rate as a human-readable speed (e.g. `1.2 MiB/s`).
fn format_rate(bytes_per_sec: f64) -> String {
    let bytes = bytes_per_sec.max(0.0) as u64;
    format!("{}/s", ByteSize::b(bytes))
}

/// Renders the `/LIMIT` suffix for a speed limit, or empty when unlimited.
/// Limits come back in KiB/s from the daemon config.
fn format_speed_limit(limit: Option<f64>) -> String {
    match limit {
        Some(l) if l > 0.0 => {
            let bytes_per_sec = (l * 1024.0) as u64;
            format!("/{}", ByteSize::b(bytes_per_sec))
        }
        _ => String::new(),
    }
}

/// Activity-first ordering for observed torrent states.
const STATE_ORDER: &[&str] = &[
    "Downloading",
    "Seeding",
    "Checking",
    "Queued",
    "Paused",
    "Error",
    "Allocating",
    "Moving",
];

/// Returns observed states ordered by activity priority, with any unknown
/// states appended alphabetically at the end.
fn order_states<'a>(states: impl Iterator<Item = &'a String>) -> Vec<&'a str> {
    let known: HashSet<&str> = states.map(|s| s.as_str()).collect();
    let mut ordered: Vec<&str> = STATE_ORDER
        .iter()
        .filter(|s| known.contains(*s))
        .copied()
        .collect();
    let mut unknown: Vec<&str> = known
        .iter()
        .filter(|s| !STATE_ORDER.contains(s))
        .copied()
        .collect();
    unknown.sort_unstable();
    ordered.extend(unknown);
    ordered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_format_rate_zero_then_zero_speed() {
        assert_eq!(format_rate(0.0), "0 B/s");
    }

    #[test]
    fn when_format_rate_kib_then_kib_per_sec() {
        assert_eq!(format_rate(1024.0), "1.0 KiB/s");
    }

    #[test]
    fn when_format_rate_negative_then_zero() {
        assert_eq!(format_rate(-100.0), "0 B/s");
    }

    #[test]
    fn when_format_speed_limit_none_then_empty() {
        assert_eq!(format_speed_limit(None), "");
    }

    #[test]
    fn when_format_speed_limit_zero_then_empty() {
        assert_eq!(format_speed_limit(Some(0.0)), "");
    }

    #[test]
    fn when_format_speed_limit_negative_then_empty() {
        assert_eq!(format_speed_limit(Some(-1.0)), "");
    }

    #[test]
    fn when_format_speed_limit_positive_then_kib() {
        assert_eq!(format_speed_limit(Some(1024.0)), "/1.0 MiB");
    }

    #[test]
    fn when_order_states_known_then_activity_order() {
        let states = [
            "Paused".to_owned(),
            "Downloading".to_owned(),
            "Seeding".to_owned(),
        ];
        let ordered = order_states(states.iter());
        assert_eq!(ordered, vec!["Downloading", "Seeding", "Paused"]);
    }

    #[test]
    fn when_order_states_unknown_then_appended_alphabetical() {
        let states = [
            "Zeta".to_owned(),
            "Downloading".to_owned(),
            "Alpha".to_owned(),
        ];
        let ordered = order_states(states.iter());
        assert_eq!(ordered, vec!["Downloading", "Alpha", "Zeta"]);
    }

    #[test]
    fn when_order_states_empty_then_empty() {
        let states: Vec<String> = vec![];
        let ordered = order_states(states.iter());
        assert!(ordered.is_empty());
    }

    #[test]
    fn when_extra_i64_missing_key_then_zero() {
        let extra = HashMap::new();
        let result = extra_i64(&extra, "missing").expect("ok");
        assert_eq!(result, 0);
    }

    #[test]
    fn when_extra_i64_present_int_then_value() {
        let mut extra = HashMap::new();
        extra.insert("key".to_owned(), RencodeValue::Int(42));
        let result = extra_i64(&extra, "key").expect("ok");
        assert_eq!(result, 42);
    }
}
