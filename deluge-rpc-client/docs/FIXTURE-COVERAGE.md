# Method Coverage Checklist

112 methods total across 4 spec documents. Each row maps a spec method to its Rust trait, sub-client path, args struct,
return struct, cassette fixture, and test status.

The "Cassette" is the file name of the JSON cassette fixture in `deluge-rpc-client/fixtures`.

## daemon (7 methods)

| # | Spec method                 | Trait       | Sub-client path                             | Args                                   | Return        | Cassette | Test |
|---|-----------------------------|-------------|---------------------------------------------|----------------------------------------|---------------|----------|------|
| 1 | `daemon.info`               | `DaemonRpc` | `client.daemon.info()`                    | `()`                                   | `String`      | live-daemon | e2e  |
| 2 | `daemon.login`              | `DaemonRpc` | `client.daemon.login(u,p,v)`              | `(username, password, client_version)` | `i64`         | -        | unit |
| 3 | `daemon.set_event_interest` | `DaemonRpc` | `client.daemon.set_event_interest(names)` | `(event_names)`                        | `bool`        | daemon-methods | e2e  |
| 4 | `daemon.shutdown`           | `DaemonRpc` | `client.daemon.shutdown()`                | `()`                                   | `()`          | daemon-methods | e2e  |
| 5 | `daemon.get_method_list`    | `DaemonRpc` | `client.daemon.get_method_list()`         | `()`                                   | `Vec<String>` | daemon-methods | e2e  |
| 6 | `daemon.get_version`        | `DaemonRpc` | `client.daemon.get_version()`             | `()`                                   | `String`      | live-daemon | e2e  |
| 7 | `daemon.authorized_call`    | `DaemonRpc` | `client.daemon.authorized_call(rpc)`      | `(rpc)`                                | `bool`        | daemon-methods | e2e  |

## core torrents (30 methods)

| #  | Spec method                     | Trait            | Sub-client path                                        | Args                                        | Return                  | Cassette | Test |
|----|---------------------------------|------------------|--------------------------------------------------------|---------------------------------------------|-------------------------|----------|------|
| 8  | `core.add_torrent_file`         | `CoreTorrentRpc` | `client.core.torrents.add_torrent_file(...)`         | `(filename, filedump, options)`             | `Option<String>`        | torrent-add | e2e  |
| 9  | `core.add_torrent_file_async`   | `CoreTorrentRpc` | `client.core.torrents.add_torrent_file_async(...)`   | `(filename, filedump, options, save_state)` | `Option<String>`        | torrent-add | e2e  |
| 10 | `core.add_torrent_files`        | `CoreTorrentRpc` | `client.core.torrents.add_torrent_files(...)`        | `(torrent_files)`                           | `Vec<AddTorrentError>`  | torrent-add | e2e  |
| 11 | `core.add_torrent_url`          | `CoreTorrentRpc` | `client.core.torrents.add_torrent_url(...)`          | `(url, options, headers)`                   | `Option<String>`        | torrent-add | e2e  |
| 12 | `core.add_torrent_magnet`       | `CoreTorrentRpc` | `client.core.torrents.add_torrent_magnet(...)`       | `(uri, options)`                            | `String`                | torrent-add | e2e  |
| 13 | `core.prefetch_magnet_metadata` | `CoreTorrentRpc` | `client.core.torrents.prefetch_magnet_metadata(...)` | `(magnet, timeout)`                         | `(String, Vec<u8>)`     | torrent-add | e2e  |
| 14 | `core.remove_torrent`           | `CoreTorrentRpc` | `client.core.torrents.remove_torrent(...)`           | `(torrent_id, remove_data)`                 | `bool`                  | torrent-lifecycle | e2e  |
| 15 | `core.remove_torrents`          | `CoreTorrentRpc` | `client.core.torrents.remove_torrents(...)`          | `(torrent_ids, remove_data)`                | `Vec<(String, String)>` | torrent-add | e2e  |
| 16 | `core.pause_torrent`            | `CoreTorrentRpc` | `client.core.torrents.pause_torrent(...)`            | `(torrent_id)`                              | `()`                    | torrent-ops | e2e  |
| 17 | `core.pause_torrents`           | `CoreTorrentRpc` | `client.core.torrents.pause_torrents(...)`           | `(torrent_ids)`                             | `()`                    | torrent-ops | e2e  |
| 18 | `core.resume_torrent`           | `CoreTorrentRpc` | `client.core.torrents.resume_torrent(...)`           | `(torrent_id)`                              | `()`                    | torrent-ops | e2e  |
| 19 | `core.resume_torrents`          | `CoreTorrentRpc` | `client.core.torrents.resume_torrents(...)`          | `(torrent_ids)`                             | `()`                    | torrent-ops | e2e  |
| 20 | `core.force_reannounce`         | `CoreTorrentRpc` | `client.core.torrents.force_reannounce(...)`         | `(torrent_ids)`                             | `()`                    | torrent-ops | e2e  |
| 21 | `core.force_recheck`            | `CoreTorrentRpc` | `client.core.torrents.force_recheck(...)`            | `(torrent_ids)`                             | `()`                    | torrent-ops | e2e  |
| 22 | `core.set_torrent_options`      | `CoreTorrentRpc` | `client.core.torrents.set_torrent_options(...)`      | `(torrent_ids, options)`                    | `()`                    | torrent-ops | e2e  |
| 23 | `core.connect_peer`             | `CoreTorrentRpc` | `client.core.torrents.connect_peer(...)`             | `(torrent_id, ip, port)`                    | `()`                    | torrent-ops | e2e  |
| 24 | `core.move_storage`             | `CoreTorrentRpc` | `client.core.torrents.move_storage(...)`             | `(torrent_ids, dest)`                       | `()`                    | torrent-ops | e2e  |
| 25 | `core.set_ssl_torrent_cert`     | `CoreTorrentRpc` | `client.core.torrents.set_ssl_torrent_cert(...)`     | `(torrent_id, cert, key, dh, save)`         | `()`                    | torrent-ops | e2e  |
| 26 | `core.get_torrent_status`       | `CoreTorrentRpc` | `client.core.torrents.get_torrent_status(...)`       | `(torrent_id, keys, diff)`                  | `TorrentStatus`         | torrent-lifecycle | e2e  |
| 27 | `core.get_torrents_status`      | `CoreTorrentRpc` | `client.core.torrents.get_torrents_status(...)`      | `(filter_dict, keys, diff)`                 | `Vec<TorrentEntry>`     | live-daemon | e2e  |
| 28 | `core.get_filter_tree`          | `CoreTorrentRpc` | `client.core.torrents.get_filter_tree(...)`          | `(show_zero_hits, hide_cat)`                | `FilterTree`            | torrent-ops | e2e  |
| 29 | `core.get_session_state`        | `CoreTorrentRpc` | `client.core.torrents.get_session_state()`           | `()`                                        | `Vec<String>`           | torrent-ops | e2e  |
| 30 | `core.get_magnet_uri`           | `CoreTorrentRpc` | `client.core.torrents.get_magnet_uri(...)`           | `(torrent_id)`                              | `String`                | torrent-lifecycle | e2e  |
| 31 | `core.get_path_size`            | `CoreTorrentRpc` | `client.core.torrents.get_path_size(...)`            | `(path)`                                    | `i64`                   | torrent-ops | e2e  |
| 32 | `core.set_torrent_trackers`     | `CoreTorrentRpc` | `client.core.torrents.set_torrent_trackers(...)`     | `(torrent_id, trackers)`                    | `()`                    | torrent-ops | e2e  |
| 33 | `core.rename_files`             | `CoreTorrentRpc` | `client.core.torrents.rename_files(...)`             | `(torrent_id, filenames)`                   | `()`                    | torrent-ops | e2e  |
| 34 | `core.rename_folder`            | `CoreTorrentRpc` | `client.core.torrents.rename_folder(...)`            | `(torrent_id, folder, new_folder)`          | `()`                    | torrent-ops | e2e  |
| 35 | `core.queue_top`                | `CoreTorrentRpc` | `client.core.torrents.queue_top(...)`                | `(torrent_ids)`                             | `()`                    | torrent-ops | e2e  |
| 36 | `core.queue_up`                 | `CoreTorrentRpc` | `client.core.torrents.queue_up(...)`                 | `(torrent_ids)`                             | `()`                    | torrent-ops | e2e  |
| 37 | `core.queue_down`               | `CoreTorrentRpc` | `client.core.torrents.queue_down(...)`               | `(torrent_ids)`                             | `()`                    | torrent-ops | e2e  |

## core session/config/network (30 methods)

| #  | Spec method                     | Trait             | Sub-client path                                     | Args                              | Return                                           | Cassette | Test |
|----|---------------------------------|-------------------|-----------------------------------------------------|-----------------------------------|--------------------------------------------------|----------|------|
| 38 | `core.pause_session`            | `CoreSessionRpc`  | `client.core.session.pause_session()`             | `()`                              | `()`                                             | session-methods | e2e  |
| 39 | `core.resume_session`           | `CoreSessionRpc`  | `client.core.session.resume_session()`            | `()`                              | `()`                                             | session-methods | e2e  |
| 40 | `core.is_session_paused`        | `CoreSessionRpc`  | `client.core.session.is_session_paused()`         | `()`                              | `bool`                                           | session-methods | e2e  |
| 41 | `core.get_listen_port`          | `CoreSessionRpc`  | `client.core.session.get_listen_port()`           | `()`                              | `i64`                                            | session-methods | e2e  |
| 42 | `core.get_ssl_listen_port`      | `CoreSessionRpc`  | `client.core.session.get_ssl_listen_port()`       | `()`                              | `i64`                                            | -        | unit |
| 43 | `core.get_external_ip`          | `CoreSessionRpc`  | `client.core.session.get_external_ip()`           | `()`                              | `String`                                         | session-methods | e2e  |
| 44 | `core.get_libtorrent_version`   | `CoreSessionRpc`  | `client.core.session.get_libtorrent_version()`    | `()`                              | `String`                                         | session-methods | e2e  |
| 45 | `core.test_listen_port`         | `CoreSessionRpc`  | `client.core.session.test_listen_port()`          | `()`                              | `Option<bool>`                                   | session-methods | e2e  |
| 46 | `core.get_session_status`       | `CoreSessionRpc`  | `client.core.session.get_session_status(keys)`    | `(keys)`                          | `SessionStatus`                                  | live-daemon | e2e  |
| 47 | `core.get_free_space`           | `CoreSessionRpc`  | `client.core.session.get_free_space(path)`        | `(path)`                          | `i64`                                            | live-daemon | e2e  |
| 48 | `core.get_config`               | `CoreConfigRpc`   | `client.core.config.get_config()`                 | `()`                              | `DaemonConfig`                                   | live-daemon | e2e  |
| 49 | `core.get_config_value`         | `CoreConfigRpc`   | `client.core.config.get_config_value(key)`        | `(key)`                           | `RencodeValue`                                   | config-methods | e2e  |
| 50 | `core.get_config_values`        | `CoreConfigRpc`   | `client.core.config.get_config_values(keys)`      | `(keys)`                          | `BTreeMap<String, RencodeValue>`                 | config-methods | e2e  |
| 51 | `core.set_config`               | `CoreConfigRpc`   | `client.core.config.set_config(config)`           | `(config)`                        | `()`                                             | config-methods | e2e  |
| 52 | `core.get_proxy`                | `CoreConfigRpc`   | `client.core.config.get_proxy()`                  | `()`                              | `ProxyConfig`                                    | config-methods | e2e  |
| 53 | `core.get_available_plugins`    | `CorePluginsRpc`  | `client.core.plugins.get_available_plugins()`     | `()`                              | `Vec<String>`                                    | plugins-manage | e2e  |
| 54 | `core.get_enabled_plugins`      | `CorePluginsRpc`  | `client.core.plugins.get_enabled_plugins()`       | `()`                              | `Vec<String>`                                    | live-daemon | e2e  |
| 55 | `core.enable_plugin`            | `CorePluginsRpc`  | `client.core.plugins.enable_plugin(plugin)`       | `(plugin)`                        | `bool`                                           | plugins-manage | e2e  |
| 56 | `core.disable_plugin`           | `CorePluginsRpc`  | `client.core.plugins.disable_plugin(plugin)`      | `(plugin)`                        | `bool`                                           | plugins-manage | e2e  |
| 57 | `core.upload_plugin`            | `CorePluginsRpc`  | `client.core.plugins.upload_plugin(...)`          | `(filename, filedump)`            | `()`                                             | -        | unit |
| 58 | `core.rescan_plugins`           | `CorePluginsRpc`  | `client.core.plugins.rescan_plugins()`            | `()`                              | `()`                                             | plugins-manage | e2e  |
| 59 | `core.get_known_accounts`       | `CoreAccountsRpc` | `client.core.accounts.get_known_accounts()`       | `()`                              | `Vec<AccountInfo>`                               | accounts | e2e  |
| 60 | `core.create_account`           | `CoreAccountsRpc` | `client.core.accounts.create_account(...)`        | `(username, password, authlevel)` | `bool`                                           | accounts | e2e  |
| 61 | `core.update_account`           | `CoreAccountsRpc` | `client.core.accounts.update_account(...)`        | `(username, password, authlevel)` | `bool`                                           | accounts | e2e  |
| 62 | `core.remove_account`           | `CoreAccountsRpc` | `client.core.accounts.remove_account(username)`   | `(username)`                      | `bool`                                           | accounts | e2e  |
| 63 | `core.get_auth_levels_mappings` | `CoreAccountsRpc` | `client.core.accounts.get_auth_levels_mappings()` | `()`                              | `(BTreeMap<String, i64>, BTreeMap<i64, String>)` | accounts | e2e  |
| 64 | `core.create_torrent`           | `CoreMiscRpc`     | `client.core.misc.create_torrent(...)`            | `(path, tracker, ...)`            | `(String, String)`                               | misc-methods | e2e  |
| 65 | `core.glob`                     | `CoreMiscRpc`     | `client.core.misc.glob(path)`                     | `(path)`                          | `Vec<String>`                                    | misc-methods | e2e  |
| 66 | `core.get_completion_paths`     | `CoreMiscRpc`     | `client.core.misc.get_completion_paths(args)`     | `(args)`                          | `CompletionPaths`                                | misc-methods | e2e  |
| 67 | `core.queue_bottom`             | `CoreTorrentRpc`  | `client.core.torrents.queue_bottom(...)`          | `(torrent_ids)`                   | `()`                                             | torrent-ops | e2e  |

## plugins (45 methods)

### autoadd (10)

| #  | Spec method                | Trait        | Sub-client path                                 | Args                     | Return                              | Cassette | Test |
|----|----------------------------|--------------|-------------------------------------------------|--------------------------|-------------------------------------|----------|------|
| 68 | `autoadd.set_options`      | `AutoaddRpc` | `client.plugins.autoadd.set_options(...)`     | `(watchdir_id, options)` | `()`                                | autoadd | e2e  |
| 69 | `autoadd.enable_watchdir`  | `AutoaddRpc` | `client.plugins.autoadd.enable_watchdir(id)`  | `(watchdir_id)`          | `()`                                | autoadd | e2e  |
| 70 | `autoadd.disable_watchdir` | `AutoaddRpc` | `client.plugins.autoadd.disable_watchdir(id)` | `(watchdir_id)`          | `()`                                | autoadd | e2e  |
| 71 | `autoadd.set_config`       | `AutoaddRpc` | `client.plugins.autoadd.set_config(config)`   | `(config)`               | `()`                                | autoadd | e2e  |
| 72 | `autoadd.get_config`       | `AutoaddRpc` | `client.plugins.autoadd.get_config()`         | `()`                     | `AutoaddConfig`                     | autoadd | e2e  |
| 73 | `autoadd.get_watchdirs`    | `AutoaddRpc` | `client.plugins.autoadd.get_watchdirs()`      | `()`                     | `BTreeMap<String, WatchdirOptions>` | autoadd | e2e  |
| 74 | `autoadd.add`              | `AutoaddRpc` | `client.plugins.autoadd.add(options)`         | `(options)`              | `i64`                               | autoadd | e2e  |
| 75 | `autoadd.remove`           | `AutoaddRpc` | `client.plugins.autoadd.remove(id)`           | `(watchdir_id)`          | `()`                                | autoadd | e2e  |
| 76 | `autoadd.is_admin_level`   | `AutoaddRpc` | `client.plugins.autoadd.is_admin_level()`     | `()`                     | `bool`                              | autoadd | e2e  |
| 77 | `autoadd.get_auth_user`    | `AutoaddRpc` | `client.plugins.autoadd.get_auth_user()`      | `()`                     | `String`                            | autoadd | e2e  |

### blocklist (4)

| #  | Spec method              | Trait          | Sub-client path                                  | Args       | Return            | Cassette | Test |
|----|--------------------------|----------------|--------------------------------------------------|------------|-------------------|----------|------|
| 78 | `blocklist.check_import` | `BlocklistRpc` | `client.plugins.blocklist.check_import(force)` | `(force)`  | `Option<String>`  | blocklist | e2e  |
| 79 | `blocklist.get_config`   | `BlocklistRpc` | `client.plugins.blocklist.get_config()`        | `()`       | `BlocklistConfig` | blocklist | e2e  |
| 80 | `blocklist.set_config`   | `BlocklistRpc` | `client.plugins.blocklist.set_config(config)`  | `(config)` | `()`              | blocklist | e2e  |
| 81 | `blocklist.get_status`   | `BlocklistRpc` | `client.plugins.blocklist.get_status()`        | `()`       | `BlocklistStatus` | blocklist | e2e  |

### execute (4)

| #  | Spec method              | Trait        | Sub-client path                               | Args                       | Return                          | Cassette | Test |
|----|--------------------------|--------------|-----------------------------------------------|----------------------------|---------------------------------|----------|------|
| 82 | `execute.add_command`    | `ExecuteRpc` | `client.plugins.execute.add_command(...)`   | `(event, command)`         | `()`                            | execute | e2e  |
| 83 | `execute.get_commands`   | `ExecuteRpc` | `client.plugins.execute.get_commands()`     | `()`                       | `Vec<(String, String, String)>` | execute | e2e  |
| 84 | `execute.remove_command` | `ExecuteRpc` | `client.plugins.execute.remove_command(id)` | `(command_id)`             | `()`                            | execute | e2e  |
| 85 | `execute.save_command`   | `ExecuteRpc` | `client.plugins.execute.save_command(...)`  | `(command_id, event, cmd)` | `()`                            | execute | e2e  |

### extractor (2)

| #  | Spec method            | Trait          | Sub-client path                                 | Args       | Return            | Cassette | Test |
|----|------------------------|----------------|-------------------------------------------------|------------|-------------------|----------|------|
| 86 | `extractor.set_config` | `ExtractorRpc` | `client.plugins.extractor.set_config(config)` | `(config)` | `()`              | extractor | e2e  |
| 87 | `extractor.get_config` | `ExtractorRpc` | `client.plugins.extractor.get_config()`       | `()`       | `ExtractorConfig` | extractor | e2e  |

### label (8)

| #  | Spec method         | Trait      | Sub-client path                                | Args                       | Return         | Cassette | Test |
|----|---------------------|------------|------------------------------------------------|----------------------------|----------------|----------|------|
| 88 | `label.get_labels`  | `LabelRpc` | `client.plugins.label.get_labels()`          | `()`                       | `Vec<String>`  | label | e2e  |
| 89 | `label.add`         | `LabelRpc` | `client.plugins.label.add(label_id)`         | `(label_id)`               | `()`           | label | e2e  |
| 90 | `label.remove`      | `LabelRpc` | `client.plugins.label.remove(label_id)`      | `(label_id)`               | `()`           | label | e2e  |
| 91 | `label.set_options` | `LabelRpc` | `client.plugins.label.set_options(...)`      | `(label_id, options_dict)` | `()`           | label | e2e  |
| 92 | `label.get_options` | `LabelRpc` | `client.plugins.label.get_options(label_id)` | `(label_id)`               | `LabelOptions` | label | e2e  |
| 93 | `label.set_torrent` | `LabelRpc` | `client.plugins.label.set_torrent(...)`      | `(torrent_id, label_id)`   | `()`           | label | e2e  |
| 94 | `label.get_config`  | `LabelRpc` | `client.plugins.label.get_config()`          | `()`                       | `LabelConfig`  | label | e2e  |
| 95 | `label.set_config`  | `LabelRpc` | `client.plugins.label.set_config(options)`   | `(options)`                | `()`           | label | e2e  |

### notifications (3)

| #  | Spec method                        | Trait              | Sub-client path                                       | Args       | Return                  | Cassette | Test |
|----|------------------------------------|--------------------|-------------------------------------------------------|------------|-------------------------|----------|------|
| 96 | `notifications.set_config`         | `NotificationsRpc` | `client.plugins.notifications.set_config(config)`   | `(config)` | `()`                    | notifications | e2e  |
| 97 | `notifications.get_config`         | `NotificationsRpc` | `client.plugins.notifications.get_config()`         | `()`       | `NotificationsConfig`   | notifications | e2e  |
| 98 | `notifications.get_handled_events` | `NotificationsRpc` | `client.plugins.notifications.get_handled_events()` | `()`       | `Vec<(String, String)>` | notifications | e2e  |

### scheduler (3)

| #   | Spec method            | Trait          | Sub-client path                                 | Args       | Return            | Cassette | Test |
|-----|------------------------|----------------|-------------------------------------------------|------------|-------------------|----------|------|
| 99  | `scheduler.set_config` | `SchedulerRpc` | `client.plugins.scheduler.set_config(config)` | `(config)` | `()`              | scheduler | e2e  |
| 100 | `scheduler.get_config` | `SchedulerRpc` | `client.plugins.scheduler.get_config()`       | `()`       | `SchedulerConfig` | scheduler | e2e  |
| 101 | `scheduler.get_state`  | `SchedulerRpc` | `client.plugins.scheduler.get_state()`        | `()`       | `String`          | scheduler | e2e  |

### stats (6)

| #   | Spec method                | Trait      | Sub-client path                               | Args               | Return                | Cassette | Test |
|-----|----------------------------|------------|-----------------------------------------------|--------------------|-----------------------|----------|------|
| 102 | `stats.get_stats`          | `StatsRpc` | `client.plugins.stats.get_stats(...)`       | `(keys, interval)` | `Option<StatsResult>` | stats | e2e  |
| 103 | `stats.get_totals`         | `StatsRpc` | `client.plugins.stats.get_totals()`         | `()`               | `StatsTotals`         | stats | e2e  |
| 104 | `stats.get_session_totals` | `StatsRpc` | `client.plugins.stats.get_session_totals()` | `()`               | `StatsTotals`         | stats | e2e  |
| 105 | `stats.set_config`         | `StatsRpc` | `client.plugins.stats.set_config(config)`   | `(config)`         | `()`                  | stats | e2e  |
| 106 | `stats.get_config`         | `StatsRpc` | `client.plugins.stats.get_config()`         | `()`               | `StatsConfig`         | stats | e2e  |
| 107 | `stats.get_intervals`      | `StatsRpc` | `client.plugins.stats.get_intervals()`      | `()`               | `Vec<i64>`            | stats | e2e  |

### toggle (2)

| #   | Spec method         | Trait       | Sub-client path                        | Args | Return | Cassette | Test |
|-----|---------------------|-------------|----------------------------------------|------|--------|----------|------|
| 108 | `toggle.get_status` | `ToggleRpc` | `client.plugins.toggle.get_status()` | `()` | `bool` | toggle | e2e  |
| 109 | `toggle.toggle`     | `ToggleRpc` | `client.plugins.toggle.toggle()`     | `()` | `bool` | toggle | e2e  |

### webui (3)

| #   | Spec method            | Trait      | Sub-client path                             | Args       | Return        | Cassette | Test |
|-----|------------------------|------------|---------------------------------------------|------------|---------------|----------|------|
| 110 | `webui.got_deluge_web` | `WebuiRpc` | `client.plugins.webui.got_deluge_web()`   | `()`       | `bool`        | webui | e2e  |
| 111 | `webui.set_config`     | `WebuiRpc` | `client.plugins.webui.set_config(config)` | `(config)` | `()`          | webui | e2e  |
| 112 | `webui.get_config`     | `WebuiRpc` | `client.plugins.webui.get_config()`       | `()`       | `WebuiConfig` | webui | e2e  |

## Summary

| Category      | Count   | Traits             | Implemented | Cassette e2e |
|---------------|---------|--------------------|-------------|--------------|
| daemon        | 7       | `DaemonRpc`        | 7/7         | 6/7          |
| core torrents | 30      | `CoreTorrentRpc`   | 30/30       | 30/30        |
| core session  | 10      | `CoreSessionRpc`   | 10/10       | 8/10         |
| core config   | 5       | `CoreConfigRpc`    | 5/5         | 5/5          |
| core plugins  | 6       | `CorePluginsRpc`   | 6/6         | 5/6          |
| core accounts | 5       | `CoreAccountsRpc`  | 5/5         | 5/5          |
| core misc     | 3       | `CoreMiscRpc`      | 3/3         | 3/3          |
| autoadd       | 10      | `AutoaddRpc`       | 10/10       | 10/10        |
| blocklist     | 4       | `BlocklistRpc`     | 4/4         | 4/4          |
| execute       | 4       | `ExecuteRpc`       | 4/4         | 4/4          |
| extractor     | 2       | `ExtractorRpc`     | 2/2         | 2/2          |
| label         | 8       | `LabelRpc`         | 8/8         | 8/8          |
| notifications | 3       | `NotificationsRpc` | 3/3         | 3/3          |
| scheduler     | 3       | `SchedulerRpc`     | 3/3         | 3/3          |
| stats         | 6       | `StatsRpc`         | 6/6         | 6/6          |
| toggle        | 2       | `ToggleRpc`        | 2/2         | 2/2          |
| webui         | 3       | `WebuiRpc`         | 3/3         | 3/3          |
| **Total**     | **112** | **18 traits**      | **112/112** | **110/112**  |
