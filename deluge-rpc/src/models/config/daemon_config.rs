use serde::{Deserialize, Serialize};

use super::proxy::ProxyConfig;
use crate::models::sentinels::{deserialize_unlimited_f64, deserialize_unlimited_i64};

/// Full daemon configuration returned by `core.get_config()`.
///
/// Contains 77 top-level keys covering info, daemon, storage, network, DHT,
/// encryption, bandwidth, queue, seeding, torrent defaults, plugins, path
/// chooser UI, updates, proxy, and miscellaneous settings.
///
/// Sentinel values (`-1` / `-1.0` meaning "unlimited") are deserialized as
/// `None` via `deserialize_unlimited_i64` / `deserialize_unlimited_f64`.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct DaemonConfig {
    // --- Info / telemetry ---
    pub send_info: bool,
    pub info_sent: f64,

    // --- Daemon / remote ---
    pub daemon_port: i64,
    pub allow_remote: bool,

    // --- Storage / file management ---
    pub pre_allocate_storage: bool,
    pub download_location: String,
    pub copy_torrent_file: bool,
    pub del_copy_torrent_file: bool,
    pub torrentfiles_location: String,
    pub plugins_location: String,
    pub move_completed: bool,
    pub move_completed_path: String,
    pub move_completed_paths_list: Vec<String>,
    pub download_location_paths_list: Vec<String>,

    // --- Network / listening ---
    pub listen_ports: Vec<i64>,
    pub listen_interface: String,
    pub outgoing_interface: String,
    pub random_port: bool,
    pub listen_random_port: Option<i64>,
    pub listen_use_sys_port: bool,
    pub listen_reuse_port: bool,
    pub ssl_torrents: bool,
    pub ssl_listen_ports: Vec<i64>,
    pub ssl_torrents_certs: String,
    pub outgoing_ports: Vec<i64>,
    pub random_outgoing_ports: bool,

    // --- DHT / PEX / LSD / trackers ---
    pub dht: bool,
    pub upnp: bool,
    pub natpmp: bool,
    pub utpex: bool,
    pub lsd: bool,
    pub announce_to_all_tiers: bool,

    // --- Encryption ---
    pub enc_in_policy: i64,
    pub enc_out_policy: i64,
    pub enc_level: i64,

    // --- Bandwidth / connections (global) ---
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_connections_global: Option<i64>,
    #[serde(deserialize_with = "deserialize_unlimited_f64")]
    pub max_upload_speed: Option<f64>,
    #[serde(deserialize_with = "deserialize_unlimited_f64")]
    pub max_download_speed: Option<f64>,
    pub max_upload_slots_global: i64,
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_half_open_connections: Option<i64>,
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_connections_per_second: Option<i64>,
    pub ignore_limits_on_local_network: bool,
    pub rate_limit_ip_overhead: bool,

    // --- Bandwidth / connections (per-torrent) ---
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_connections_per_torrent: Option<i64>,
    #[serde(deserialize_with = "deserialize_unlimited_i64")]
    pub max_upload_slots_per_torrent: Option<i64>,
    #[serde(deserialize_with = "deserialize_unlimited_f64")]
    pub max_upload_speed_per_torrent: Option<f64>,
    #[serde(deserialize_with = "deserialize_unlimited_f64")]
    pub max_download_speed_per_torrent: Option<f64>,

    // --- Queue / active management ---
    pub max_active_seeding: i64,
    pub max_active_downloading: i64,
    pub max_active_limit: i64,
    pub dont_count_slow_torrents: bool,
    pub queue_new_to_top: bool,
    pub auto_managed: bool,
    pub auto_manage_prefer_seeds: bool,

    // --- Seeding / ratio ---
    pub stop_seed_at_ratio: bool,
    pub remove_seed_at_ratio: bool,
    pub stop_seed_ratio: f64,
    pub share_ratio_limit: f64,
    pub seed_time_ratio_limit: f64,
    pub seed_time_limit: i64,

    // --- Torrent options (defaults for new torrents) ---
    pub prioritize_first_last_pieces: bool,
    pub sequential_download: bool,
    pub add_paused: bool,
    pub super_seeding: bool,
    pub shared: bool,

    // --- Plugins ---
    pub enabled_plugins: Vec<String>,

    // --- Path chooser UI ---
    pub path_chooser_show_chooser_button_on_localhost: bool,
    pub path_chooser_auto_complete_enabled: bool,
    pub path_chooser_accelerator_string: String,
    pub path_chooser_max_popup_rows: i64,
    pub path_chooser_show_hidden_files: bool,

    // --- Updates ---
    pub new_release_check: bool,

    // --- Proxy (nested dict) ---
    pub proxy: ProxyConfig,

    // --- Miscellaneous ---
    pub peer_tos: String,
    pub geoip_db_location: String,
    pub cache_size: i64,
    pub cache_expiry: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    fn make_full_config_dict() -> RencodeValue {
        let mut map = BTreeMap::new();

        // Info / telemetry
        map.insert(
            RencodeValue::Str("send_info".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("info_sent".into()),
            RencodeValue::Float(0.0),
        );

        // Daemon / remote
        map.insert(
            RencodeValue::Str("daemon_port".into()),
            RencodeValue::Int(58846),
        );
        map.insert(
            RencodeValue::Str("allow_remote".into()),
            RencodeValue::Bool(false),
        );

        // Storage
        map.insert(
            RencodeValue::Str("pre_allocate_storage".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("download_location".into()),
            RencodeValue::Str("/downloads".into()),
        );
        map.insert(
            RencodeValue::Str("copy_torrent_file".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("del_copy_torrent_file".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("torrentfiles_location".into()),
            RencodeValue::Str("/downloads".into()),
        );
        map.insert(
            RencodeValue::Str("plugins_location".into()),
            RencodeValue::Str("/config/plugins".into()),
        );
        map.insert(
            RencodeValue::Str("move_completed".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("move_completed_path".into()),
            RencodeValue::Str("/downloads".into()),
        );
        map.insert(
            RencodeValue::Str("move_completed_paths_list".into()),
            RencodeValue::List(vec![]),
        );
        map.insert(
            RencodeValue::Str("download_location_paths_list".into()),
            RencodeValue::List(vec![]),
        );

        // Network
        map.insert(
            RencodeValue::Str("listen_ports".into()),
            RencodeValue::List(vec![RencodeValue::Int(6881), RencodeValue::Int(6891)]),
        );
        map.insert(
            RencodeValue::Str("listen_interface".into()),
            RencodeValue::Str("".into()),
        );
        map.insert(
            RencodeValue::Str("outgoing_interface".into()),
            RencodeValue::Str("".into()),
        );
        map.insert(
            RencodeValue::Str("random_port".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("listen_random_port".into()),
            RencodeValue::None,
        );
        map.insert(
            RencodeValue::Str("listen_use_sys_port".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("listen_reuse_port".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("ssl_torrents".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("ssl_listen_ports".into()),
            RencodeValue::List(vec![RencodeValue::Int(6892), RencodeValue::Int(6896)]),
        );
        map.insert(
            RencodeValue::Str("ssl_torrents_certs".into()),
            RencodeValue::Str("/config/ssl".into()),
        );
        map.insert(
            RencodeValue::Str("outgoing_ports".into()),
            RencodeValue::List(vec![RencodeValue::Int(0), RencodeValue::Int(0)]),
        );
        map.insert(
            RencodeValue::Str("random_outgoing_ports".into()),
            RencodeValue::Bool(true),
        );

        // DHT / PEX / LSD
        map.insert(RencodeValue::Str("dht".into()), RencodeValue::Bool(true));
        map.insert(RencodeValue::Str("upnp".into()), RencodeValue::Bool(true));
        map.insert(RencodeValue::Str("natpmp".into()), RencodeValue::Bool(true));
        map.insert(RencodeValue::Str("utpex".into()), RencodeValue::Bool(true));
        map.insert(RencodeValue::Str("lsd".into()), RencodeValue::Bool(true));
        map.insert(
            RencodeValue::Str("announce_to_all_tiers".into()),
            RencodeValue::Bool(false),
        );

        // Encryption
        map.insert(
            RencodeValue::Str("enc_in_policy".into()),
            RencodeValue::Int(1),
        );
        map.insert(
            RencodeValue::Str("enc_out_policy".into()),
            RencodeValue::Int(1),
        );
        map.insert(RencodeValue::Str("enc_level".into()), RencodeValue::Int(2));

        // Bandwidth global
        map.insert(
            RencodeValue::Str("max_connections_global".into()),
            RencodeValue::Int(200),
        );
        map.insert(
            RencodeValue::Str("max_upload_speed".into()),
            RencodeValue::Float(-1.0),
        );
        map.insert(
            RencodeValue::Str("max_download_speed".into()),
            RencodeValue::Float(-1.0),
        );
        map.insert(
            RencodeValue::Str("max_upload_slots_global".into()),
            RencodeValue::Int(4),
        );
        map.insert(
            RencodeValue::Str("max_half_open_connections".into()),
            RencodeValue::Int(50),
        );
        map.insert(
            RencodeValue::Str("max_connections_per_second".into()),
            RencodeValue::Int(20),
        );
        map.insert(
            RencodeValue::Str("ignore_limits_on_local_network".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("rate_limit_ip_overhead".into()),
            RencodeValue::Bool(true),
        );

        // Bandwidth per-torrent
        map.insert(
            RencodeValue::Str("max_connections_per_torrent".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_upload_slots_per_torrent".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_upload_speed_per_torrent".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_download_speed_per_torrent".into()),
            RencodeValue::Int(-1),
        );

        // Queue
        map.insert(
            RencodeValue::Str("max_active_seeding".into()),
            RencodeValue::Int(5),
        );
        map.insert(
            RencodeValue::Str("max_active_downloading".into()),
            RencodeValue::Int(3),
        );
        map.insert(
            RencodeValue::Str("max_active_limit".into()),
            RencodeValue::Int(8),
        );
        map.insert(
            RencodeValue::Str("dont_count_slow_torrents".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("queue_new_to_top".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("auto_managed".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("auto_manage_prefer_seeds".into()),
            RencodeValue::Bool(false),
        );

        // Seeding
        map.insert(
            RencodeValue::Str("stop_seed_at_ratio".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("remove_seed_at_ratio".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("stop_seed_ratio".into()),
            RencodeValue::Float(2.0),
        );
        map.insert(
            RencodeValue::Str("share_ratio_limit".into()),
            RencodeValue::Float(2.0),
        );
        map.insert(
            RencodeValue::Str("seed_time_ratio_limit".into()),
            RencodeValue::Float(7.0),
        );
        map.insert(
            RencodeValue::Str("seed_time_limit".into()),
            RencodeValue::Int(180),
        );

        // Torrent options
        map.insert(
            RencodeValue::Str("prioritize_first_last_pieces".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("sequential_download".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("add_paused".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("super_seeding".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("shared".into()),
            RencodeValue::Bool(false),
        );

        // Plugins
        map.insert(
            RencodeValue::Str("enabled_plugins".into()),
            RencodeValue::List(vec![]),
        );

        // Path chooser
        map.insert(
            RencodeValue::Str("path_chooser_show_chooser_button_on_localhost".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("path_chooser_auto_complete_enabled".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("path_chooser_accelerator_string".into()),
            RencodeValue::Str("Tab".into()),
        );
        map.insert(
            RencodeValue::Str("path_chooser_max_popup_rows".into()),
            RencodeValue::Int(20),
        );
        map.insert(
            RencodeValue::Str("path_chooser_show_hidden_files".into()),
            RencodeValue::Bool(false),
        );

        // Updates
        map.insert(
            RencodeValue::Str("new_release_check".into()),
            RencodeValue::Bool(true),
        );

        // Proxy (nested dict)
        let mut proxy_map = BTreeMap::new();
        proxy_map.insert(RencodeValue::Str("type".into()), RencodeValue::Int(0));
        proxy_map.insert(
            RencodeValue::Str("hostname".into()),
            RencodeValue::Str("".into()),
        );
        proxy_map.insert(
            RencodeValue::Str("username".into()),
            RencodeValue::Str("".into()),
        );
        proxy_map.insert(
            RencodeValue::Str("password".into()),
            RencodeValue::Str("".into()),
        );
        proxy_map.insert(RencodeValue::Str("port".into()), RencodeValue::Int(8080));
        proxy_map.insert(
            RencodeValue::Str("proxy_hostnames".into()),
            RencodeValue::Bool(true),
        );
        proxy_map.insert(
            RencodeValue::Str("proxy_peer_connections".into()),
            RencodeValue::Bool(true),
        );
        proxy_map.insert(
            RencodeValue::Str("proxy_tracker_connections".into()),
            RencodeValue::Bool(true),
        );
        proxy_map.insert(
            RencodeValue::Str("force_proxy".into()),
            RencodeValue::Bool(false),
        );
        proxy_map.insert(
            RencodeValue::Str("anonymous_mode".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("proxy".into()),
            RencodeValue::Dict(proxy_map),
        );

        // Miscellaneous
        map.insert(
            RencodeValue::Str("peer_tos".into()),
            RencodeValue::Str("0x00".into()),
        );
        map.insert(
            RencodeValue::Str("geoip_db_location".into()),
            RencodeValue::Str("/usr/share/GeoIP/GeoIP.dat".into()),
        );
        map.insert(
            RencodeValue::Str("cache_size".into()),
            RencodeValue::Int(512),
        );
        map.insert(
            RencodeValue::Str("cache_expiry".into()),
            RencodeValue::Int(60),
        );

        RencodeValue::Dict(map)
    }

    #[test]
    fn when_full_config_dict_then_all_keys_populate() {
        let value = make_full_config_dict();

        let result: DaemonConfig = DaemonConfig::deserialize(&value).expect("deserialize");

        assert!(!result.send_info);
        assert!((result.info_sent - 0.0).abs() < f64::EPSILON);
        assert_eq!(result.daemon_port, 58846);
        assert!(!result.allow_remote);
        assert!(!result.pre_allocate_storage);
        assert_eq!(result.download_location, "/downloads");
        assert!(!result.copy_torrent_file);
        assert!(!result.del_copy_torrent_file);
        assert_eq!(result.torrentfiles_location, "/downloads");
        assert_eq!(result.plugins_location, "/config/plugins");
        assert!(!result.move_completed);
        assert_eq!(result.move_completed_path, "/downloads");
        assert!(result.move_completed_paths_list.is_empty());
        assert!(result.download_location_paths_list.is_empty());
        assert_eq!(result.listen_ports, vec![6881, 6891]);
        assert_eq!(result.listen_interface, "");
        assert_eq!(result.outgoing_interface, "");
        assert!(result.random_port);
        assert_eq!(result.listen_random_port, None);
        assert!(!result.listen_use_sys_port);
        assert!(result.listen_reuse_port);
        assert!(!result.ssl_torrents);
        assert_eq!(result.ssl_listen_ports, vec![6892, 6896]);
        assert_eq!(result.ssl_torrents_certs, "/config/ssl");
        assert_eq!(result.outgoing_ports, vec![0, 0]);
        assert!(result.random_outgoing_ports);
        assert!(result.dht);
        assert!(result.upnp);
        assert!(result.natpmp);
        assert!(result.utpex);
        assert!(result.lsd);
        assert!(!result.announce_to_all_tiers);
        assert_eq!(result.enc_in_policy, 1);
        assert_eq!(result.enc_out_policy, 1);
        assert_eq!(result.enc_level, 2);
        assert_eq!(result.max_connections_global, Some(200));
        assert_eq!(result.max_upload_speed, None);
        assert_eq!(result.max_download_speed, None);
        assert_eq!(result.max_upload_slots_global, 4);
        assert_eq!(result.max_half_open_connections, Some(50));
        assert_eq!(result.max_connections_per_second, Some(20));
        assert!(result.ignore_limits_on_local_network);
        assert!(result.rate_limit_ip_overhead);
        assert_eq!(result.max_connections_per_torrent, None);
        assert_eq!(result.max_upload_slots_per_torrent, None);
        assert_eq!(result.max_upload_speed_per_torrent, None);
        assert_eq!(result.max_download_speed_per_torrent, None);
        assert_eq!(result.max_active_seeding, 5);
        assert_eq!(result.max_active_downloading, 3);
        assert_eq!(result.max_active_limit, 8);
        assert!(!result.dont_count_slow_torrents);
        assert!(!result.queue_new_to_top);
        assert!(result.auto_managed);
        assert!(!result.auto_manage_prefer_seeds);
        assert!(!result.stop_seed_at_ratio);
        assert!(!result.remove_seed_at_ratio);
        assert!((result.stop_seed_ratio - 2.0).abs() < f64::EPSILON);
        assert!((result.share_ratio_limit - 2.0).abs() < f64::EPSILON);
        assert!((result.seed_time_ratio_limit - 7.0).abs() < f64::EPSILON);
        assert_eq!(result.seed_time_limit, 180);
        assert!(!result.prioritize_first_last_pieces);
        assert!(!result.sequential_download);
        assert!(!result.add_paused);
        assert!(!result.super_seeding);
        assert!(!result.shared);
        assert!(result.enabled_plugins.is_empty());
        assert!(result.path_chooser_show_chooser_button_on_localhost);
        assert!(result.path_chooser_auto_complete_enabled);
        assert_eq!(result.path_chooser_accelerator_string, "Tab");
        assert_eq!(result.path_chooser_max_popup_rows, 20);
        assert!(!result.path_chooser_show_hidden_files);
        assert!(result.new_release_check);
        assert_eq!(result.proxy.proxy_type, 0);
        assert_eq!(result.proxy.hostname, "");
        assert_eq!(result.proxy.username, "");
        assert_eq!(result.proxy.password, "");
        assert_eq!(result.proxy.port, 8080);
        assert!(result.proxy.proxy_hostnames);
        assert!(result.proxy.proxy_peer_connections);
        assert!(result.proxy.proxy_tracker_connections);
        assert!(!result.proxy.force_proxy);
        assert!(!result.proxy.anonymous_mode);
        assert_eq!(result.peer_tos, "0x00");
        assert_eq!(result.geoip_db_location, "/usr/share/GeoIP/GeoIP.dat");
        assert_eq!(result.cache_size, 512);
        assert_eq!(result.cache_expiry, 60);
    }

    #[test]
    fn when_max_speed_minus_one_then_none() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("max_upload_speed".into()),
            RencodeValue::Float(-1.0),
        );
        map.insert(
            RencodeValue::Str("max_download_speed".into()),
            RencodeValue::Float(-1.0),
        );
        map.insert(
            RencodeValue::Str("max_connections_global".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_connections_per_torrent".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_upload_slots_per_torrent".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_upload_speed_per_torrent".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_download_speed_per_torrent".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_half_open_connections".into()),
            RencodeValue::Int(-1),
        );
        map.insert(
            RencodeValue::Str("max_connections_per_second".into()),
            RencodeValue::Int(-1),
        );
        let value = RencodeValue::Dict(map);

        #[derive(Debug, Deserialize, PartialEq)]
        struct SentinelTest {
            #[serde(deserialize_with = "deserialize_unlimited_f64")]
            max_upload_speed: Option<f64>,
            #[serde(deserialize_with = "deserialize_unlimited_f64")]
            max_download_speed: Option<f64>,
            #[serde(deserialize_with = "deserialize_unlimited_i64")]
            max_connections_global: Option<i64>,
            #[serde(deserialize_with = "deserialize_unlimited_i64")]
            max_connections_per_torrent: Option<i64>,
            #[serde(deserialize_with = "deserialize_unlimited_i64")]
            max_upload_slots_per_torrent: Option<i64>,
            #[serde(deserialize_with = "deserialize_unlimited_f64")]
            max_upload_speed_per_torrent: Option<f64>,
            #[serde(deserialize_with = "deserialize_unlimited_f64")]
            max_download_speed_per_torrent: Option<f64>,
            #[serde(deserialize_with = "deserialize_unlimited_i64")]
            max_half_open_connections: Option<i64>,
            #[serde(deserialize_with = "deserialize_unlimited_i64")]
            max_connections_per_second: Option<i64>,
        }

        let result: SentinelTest = SentinelTest::deserialize(&value).expect("deserialize");

        assert_eq!(result.max_upload_speed, None);
        assert_eq!(result.max_download_speed, None);
        assert_eq!(result.max_connections_global, None);
        assert_eq!(result.max_connections_per_torrent, None);
        assert_eq!(result.max_upload_slots_per_torrent, None);
        assert_eq!(result.max_upload_speed_per_torrent, None);
        assert_eq!(result.max_download_speed_per_torrent, None);
        assert_eq!(result.max_half_open_connections, None);
        assert_eq!(result.max_connections_per_second, None);
    }

    #[test]
    fn when_empty_config_dict_then_all_fields_default() {
        let empty = RencodeValue::Dict(BTreeMap::new());

        let result: DaemonConfig = DaemonConfig::deserialize(&empty).expect("deserialize");

        assert!(!result.send_info);
        assert!((result.info_sent - 0.0).abs() < f64::EPSILON);
        assert_eq!(result.daemon_port, 0);
        assert!(!result.allow_remote);
        assert!(!result.ssl_torrents);
        assert_eq!(result.listen_ports, Vec::<i64>::new());
        assert_eq!(result.download_location, "");
        assert_eq!(result.proxy.proxy_type, 0);
    }
}
