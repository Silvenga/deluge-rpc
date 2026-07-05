# Method Coverage Checklist

112 methods total across 4 spec documents. Each row maps a spec method to its Rust trait, sub-client path, args struct,
return struct, cassette fixture, and test status.

## daemon (7 methods)

| # | Spec method                 | Trait       | Sub-client path                             | Args                                   | Return        | Cassette                | Test       |
|---|-----------------------------|-------------|---------------------------------------------|----------------------------------------|---------------|-------------------------|------------|
| 1 | `daemon.info`               | `DaemonRpc` | `client.daemon().info()`                    | `()`                                   | `String`      | —                       | unit       |
| 2 | `daemon.login`              | `DaemonRpc` | `client.daemon().login(u,p,v)`              | `(username, password, client_version)` | `i64`         | `login_ok`, `login_bad` | unit + e2e |
| 3 | `daemon.set_event_interest` | `DaemonRpc` | `client.daemon().set_event_interest(names)` | `(event_names)`                        | `bool`        | —                       | unit       |
| 4 | `daemon.shutdown`           | `DaemonRpc` | `client.daemon().shutdown()`                | `()`                                   | `()`          | —                       | unit       |
| 5 | `daemon.get_method_list`    | `DaemonRpc` | `client.daemon().get_method_list()`         | `()`                                   | `Vec<String>` | —                       | unit       |
| 6 | `daemon.get_version`        | `DaemonRpc` | `client.daemon().get_version()`             | `()`                                   | `String`      | —                       | unit       |
| 7 | `daemon.authorized_call`    | `DaemonRpc` | `client.daemon().authorized_call(rpc)`      | `(rpc)`                                | `bool`        | —                       | unit       |

## core torrents (30 methods)

| #  | Spec method                     | Trait            | Sub-client path                                        | Args                                        | Return                  | Cassette         | Test       |
|----|---------------------------------|------------------|--------------------------------------------------------|---------------------------------------------|-------------------------|------------------|------------|
| 8  | `core.add_torrent_file`         | `CoreTorrentRpc` | `client.core().torrents.add_torrent_file(...)`         | `(filename, filedump, options)`             | `Option<String>`        | —                | unit       |
| 9  | `core.add_torrent_file_async`   | `CoreTorrentRpc` | `client.core().torrents.add_torrent_file_async(...)`   | `(filename, filedump, options, save_state)` | `Option<String>`        | —                | unit       |
| 10 | `core.add_torrent_files`        | `CoreTorrentRpc` | `client.core().torrents.add_torrent_files(...)`        | `(torrent_files)`                           | `Vec<AddTorrentError>`  | —                | unit       |
| 11 | `core.add_torrent_url`          | `CoreTorrentRpc` | `client.core().torrents.add_torrent_url(...)`          | `(url, options, headers)`                   | `Option<String>`        | —                | unit       |
| 12 | `core.add_torrent_magnet`       | `CoreTorrentRpc` | `client.core().torrents.add_torrent_magnet(...)`       | `(uri, options)`                            | `String`                | —                | unit       |
| 13 | `core.prefetch_magnet_metadata` | `CoreTorrentRpc` | `client.core().torrents.prefetch_magnet_metadata(...)` | `(magnet, timeout)`                         | `(String, Vec<u8>)`     | —                | unit       |
| 14 | `core.remove_torrent`           | `CoreTorrentRpc` | `client.core().torrents.remove_torrent(...)`           | `(torrent_id, remove_data)`                 | `bool`                  | `remove_torrent` | unit + e2e |
| 15 | `core.remove_torrents`          | `CoreTorrentRpc` | `client.core().torrents.remove_torrents(...)`          | `(torrent_ids, remove_data)`                | `Vec<(String, String)>` | —                | unit       |
| 16 | `core.pause_torrent`            | `CoreTorrentRpc` | `client.core().torrents.pause_torrent(...)`            | `(torrent_id)`                              | `()`                    | —                | unit       |
| 17 | `core.pause_torrents`           | `CoreTorrentRpc` | `client.core().torrents.pause_torrents(...)`           | `(torrent_ids)`                             | `()`                    | —                | unit       |
| 18 | `core.resume_torrent`           | `CoreTorrentRpc` | `client.core().torrents.resume_torrent(...)`           | `(torrent_id)`                              | `()`                    | —                | unit       |
| 19 | `core.resume_torrents`          | `CoreTorrentRpc` | `client.core().torrents.resume_torrents(...)`          | `(torrent_ids)`                             | `()`                    | —                | unit       |
| 20 | `core.force_reannounce`         | `CoreTorrentRpc` | `client.core().torrents.force_reannounce(...)`         | `(torrent_ids)`                             | `()`                    | —                | unit       |
| 21 | `core.force_recheck`            | `CoreTorrentRpc` | `client.core().torrents.force_recheck(...)`            | `(torrent_ids)`                             | `()`                    | —                | unit       |
| 22 | `core.set_torrent_options`      | `CoreTorrentRpc` | `client.core().torrents.set_torrent_options(...)`      | `(torrent_ids, options)`                    | `()`                    | —                | unit       |
| 23 | `core.connect_peer`             | `CoreTorrentRpc` | `client.core().torrents.connect_peer(...)`             | `(torrent_id, ip, port)`                    | `()`                    | —                | unit       |
| 24 | `core.move_storage`             | `CoreTorrentRpc` | `client.core().torrents.move_storage(...)`             | `(torrent_ids, dest)`                       | `()`                    | —                | unit       |
| 25 | `core.set_ssl_torrent_cert`     | `CoreTorrentRpc` | `client.core().torrents.set_ssl_torrent_cert(...)`     | `(torrent_id, cert, key, dh, save)`         | `()`                    | —                | unit       |
| 26 | `core.get_torrent_status`       | `CoreTorrentRpc` | `client.core().torrents.get_torrent_status(...)`       | `(torrent_id, keys, diff)`                  | `TorrentStatus`         | —                | unit       |
| 27 | `core.get_torrents_status`      | `CoreTorrentRpc` | `client.core().torrents.get_torrents_status(...)`      | `(filter_dict, keys, diff)`                 | `Vec<TorrentEntry>`     | `torrents_list`  | unit + e2e |
| 28 | `core.get_filter_tree`          | `CoreTorrentRpc` | `client.core().torrents.get_filter_tree(...)`          | `(show_zero_hits, hide_cat)`                | `FilterTree`            | —                | unit       |
| 29 | `core.get_session_state`        | `CoreTorrentRpc` | `client.core().torrents.get_session_state()`           | `()`                                        | `Vec<String>`           | —                | unit       |
| 30 | `core.get_magnet_uri`           | `CoreTorrentRpc` | `client.core().torrents.get_magnet_uri(...)`           | `(torrent_id)`                              | `String`                | —                | unit       |
| 31 | `core.get_path_size`            | `CoreTorrentRpc` | `client.core().torrents.get_path_size(...)`            | `(path)`                                    | `i64`                   | —                | unit       |
| 32 | `core.set_torrent_trackers`     | `CoreTorrentRpc` | `client.core().torrents.set_torrent_trackers(...)`     | `(torrent_id, trackers)`                    | `()`                    | —                | unit       |
| 33 | `core.rename_files`             | `CoreTorrentRpc` | `client.core().torrents.rename_files(...)`             | `(torrent_id, filenames)`                   | `()`                    | —                | unit       |
| 34 | `core.rename_folder`            | `CoreTorrentRpc` | `client.core().torrents.rename_folder(...)`            | `(torrent_id, folder, new_folder)`          | `()`                    | —                | unit       |
| 35 | `core.queue_top`                | `CoreTorrentRpc` | `client.core().torrents.queue_top(...)`                | `(torrent_ids)`                             | `()`                    | —                | unit       |
| 36 | `core.queue_up`                 | `CoreTorrentRpc` | `client.core().torrents.queue_up(...)`                 | `(torrent_ids)`                             | `()`                    | —                | unit       |
| 37 | `core.queue_down`               | `CoreTorrentRpc` | `client.core().torrents.queue_down(...)`               | `(torrent_ids)`                             | `()`                    | —                | unit       |

## core session/config/network (30 methods)

| #  | Spec method                     | Trait             | Sub-client path                                     | Args                              | Return                                           | Cassette                            | Test       |
|----|---------------------------------|-------------------|-----------------------------------------------------|-----------------------------------|--------------------------------------------------|-------------------------------------|------------|
| 38 | `core.pause_session`            | `CoreSessionRpc`  | `client.core().session.pause_session()`             | `()`                              | `()`                                             | —                                   | unit       |
| 39 | `core.resume_session`           | `CoreSessionRpc`  | `client.core().session.resume_session()`            | `()`                              | `()`                                             | —                                   | unit       |
| 40 | `core.is_session_paused`        | `CoreSessionRpc`  | `client.core().session.is_session_paused()`         | `()`                              | `bool`                                           | —                                   | unit       |
| 41 | `core.get_listen_port`          | `CoreSessionRpc`  | `client.core().session.get_listen_port()`           | `()`                              | `i64`                                            | —                                   | unit       |
| 42 | `core.get_ssl_listen_port`      | `CoreSessionRpc`  | `client.core().session.get_ssl_listen_port()`       | `()`                              | `i64`                                            | —                                   | unit       |
| 43 | `core.get_external_ip`          | `CoreSessionRpc`  | `client.core().session.get_external_ip()`           | `()`                              | `String`                                         | —                                   | unit       |
| 44 | `core.get_libtorrent_version`   | `CoreSessionRpc`  | `client.core().session.get_libtorrent_version()`    | `()`                              | `String`                                         | —                                   | unit       |
| 45 | `core.test_listen_port`         | `CoreSessionRpc`  | `client.core().session.test_listen_port()`          | `()`                              | `Option<bool>`                                   | —                                   | unit       |
| 46 | `core.get_session_status`       | `CoreSessionRpc`  | `client.core().session.get_session_status(keys)`    | `(keys)`                          | `SessionStatus`                                  | —                                   | unit       |
| 47 | `core.get_free_space`           | `CoreSessionRpc`  | `client.core().session.get_free_space(path)`        | `(path)`                          | `i64`                                            | `free_space_low`, `free_space_high` | unit + e2e |
| 48 | `core.get_config`               | `CoreConfigRpc`   | `client.core().config.get_config()`                 | `()`                              | `DaemonConfig`                                   | —                                   | unit       |
| 49 | `core.get_config_value`         | `CoreConfigRpc`   | `client.core().config.get_config_value(key)`        | `(key)`                           | `RencodeValue`                                   | —                                   | unit       |
| 50 | `core.get_config_values`        | `CoreConfigRpc`   | `client.core().config.get_config_values(keys)`      | `(keys)`                          | `BTreeMap<String, RencodeValue>`                 | —                                   | unit       |
| 51 | `core.set_config`               | `CoreConfigRpc`   | `client.core().config.set_config(config)`           | `(config)`                        | `()`                                             | —                                   | unit       |
| 52 | `core.get_proxy`                | `CoreConfigRpc`   | `client.core().config.get_proxy()`                  | `()`                              | `ProxyConfig`                                    | —                                   | unit       |
| 53 | `core.get_available_plugins`    | `CorePluginsRpc`  | `client.core().plugins.get_available_plugins()`     | `()`                              | `Vec<String>`                                    | —                                   | unit       |
| 54 | `core.get_enabled_plugins`      | `CorePluginsRpc`  | `client.core().plugins.get_enabled_plugins()`       | `()`                              | `Vec<String>`                                    | —                                   | unit       |
| 55 | `core.enable_plugin`            | `CorePluginsRpc`  | `client.core().plugins.enable_plugin(plugin)`       | `(plugin)`                        | `bool`                                           | —                                   | unit       |
| 56 | `core.disable_plugin`           | `CorePluginsRpc`  | `client.core().plugins.disable_plugin(plugin)`      | `(plugin)`                        | `bool`                                           | —                                   | unit       |
| 57 | `core.upload_plugin`            | `CorePluginsRpc`  | `client.core().plugins.upload_plugin(...)`          | `(filename, filedump)`            | `()`                                             | —                                   | unit       |
| 58 | `core.rescan_plugins`           | `CorePluginsRpc`  | `client.core().plugins.rescan_plugins()`            | `()`                              | `()`                                             | —                                   | unit       |
| 59 | `core.get_known_accounts`       | `CoreAccountsRpc` | `client.core().accounts.get_known_accounts()`       | `()`                              | `Vec<AccountInfo>`                               | —                                   | unit       |
| 60 | `core.create_account`           | `CoreAccountsRpc` | `client.core().accounts.create_account(...)`        | `(username, password, authlevel)` | `bool`                                           | —                                   | unit       |
| 61 | `core.update_account`           | `CoreAccountsRpc` | `client.core().accounts.update_account(...)`        | `(username, password, authlevel)` | `bool`                                           | —                                   | unit       |
| 62 | `core.remove_account`           | `CoreAccountsRpc` | `client.core().accounts.remove_account(username)`   | `(username)`                      | `bool`                                           | —                                   | unit       |
| 63 | `core.get_auth_levels_mappings` | `CoreAccountsRpc` | `client.core().accounts.get_auth_levels_mappings()` | `()`                              | `(BTreeMap<String, i64>, BTreeMap<i64, String>)` | —                                   | unit       |
| 64 | `core.create_torrent`           | `CoreMiscRpc`     | `client.core().misc.create_torrent(...)`            | `(path, tracker, ...)`            | `(String, String)`                               | —                                   | unit       |
| 65 | `core.glob`                     | `CoreMiscRpc`     | `client.core().misc.glob(path)`                     | `(path)`                          | `Vec<String>`                                    | —                                   | unit       |
| 66 | `core.get_completion_paths`     | `CoreMiscRpc`     | `client.core().misc.get_completion_paths(args)`     | `(args)`                          | `CompletionPaths`                                | —                                   | unit       |
| 67 | `core.queue_bottom`             | `CoreTorrentRpc`  | `client.core().torrents.queue_bottom(...)`          | `(torrent_ids)`                   | `()`                                             | —                                   | unit       |

## plugins (45 methods)

### autoadd (10)

| #  | Spec method                | Trait        | Sub-client path                                 | Args                     | Return                              | Cassette | Test |
|----|----------------------------|--------------|-------------------------------------------------|--------------------------|-------------------------------------|----------|------|
| 68 | `autoadd.set_options`      | `AutoaddRpc` | `client.plugins().autoadd.set_options(...)`     | `(watchdir_id, options)` | `()`                                | —        | unit |
| 69 | `autoadd.enable_watchdir`  | `AutoaddRpc` | `client.plugins().autoadd.enable_watchdir(id)`  | `(watchdir_id)`          | `()`                                | —        | unit |
| 70 | `autoadd.disable_watchdir` | `AutoaddRpc` | `client.plugins().autoadd.disable_watchdir(id)` | `(watchdir_id)`          | `()`                                | —        | unit |
| 71 | `autoadd.set_config`       | `AutoaddRpc` | `client.plugins().autoadd.set_config(config)`   | `(config)`               | `()`                                | —        | unit |
| 72 | `autoadd.get_config`       | `AutoaddRpc` | `client.plugins().autoadd.get_config()`         | `()`                     | `AutoaddConfig`                     | —        | unit |
| 73 | `autoadd.get_watchdirs`    | `AutoaddRpc` | `client.plugins().autoadd.get_watchdirs()`      | `()`                     | `BTreeMap<String, WatchdirOptions>` | —        | unit |
| 74 | `autoadd.add`              | `AutoaddRpc` | `client.plugins().autoadd.add(options)`         | `(options)`              | `i64`                               | —        | unit |
| 75 | `autoadd.remove`           | `AutoaddRpc` | `client.plugins().autoadd.remove(id)`           | `(watchdir_id)`          | `()`                                | —        | unit |
| 76 | `autoadd.is_admin_level`   | `AutoaddRpc` | `client.plugins().autoadd.is_admin_level()`     | `()`                     | `bool`                              | —        | unit |
| 77 | `autoadd.get_auth_user`    | `AutoaddRpc` | `client.plugins().autoadd.get_auth_user()`      | `()`                     | `String`                            | —        | unit |

### blocklist (4)

| #  | Spec method              | Trait          | Sub-client path                                  | Args       | Return            | Cassette | Test |
|----|--------------------------|----------------|--------------------------------------------------|------------|-------------------|----------|------|
| 78 | `blocklist.check_import` | `BlocklistRpc` | `client.plugins().blocklist.check_import(force)` | `(force)`  | `Option<String>`  | —        | unit |
| 79 | `blocklist.get_config`   | `BlocklistRpc` | `client.plugins().blocklist.get_config()`        | `()`       | `BlocklistConfig` | —        | unit |
| 80 | `blocklist.set_config`   | `BlocklistRpc` | `client.plugins().blocklist.set_config(config)`  | `(config)` | `()`              | —        | unit |
| 81 | `blocklist.get_status`   | `BlocklistRpc` | `client.plugins().blocklist.get_status()`        | `()`       | `BlocklistStatus` | —        | unit |

### execute (4)

| #  | Spec method              | Trait        | Sub-client path                               | Args                       | Return                          | Cassette | Test |
|----|--------------------------|--------------|-----------------------------------------------|----------------------------|---------------------------------|----------|------|
| 82 | `execute.add_command`    | `ExecuteRpc` | `client.plugins().execute.add_command(...)`   | `(event, command)`         | `()`                            | —        | unit |
| 83 | `execute.get_commands`   | `ExecuteRpc` | `client.plugins().execute.get_commands()`     | `()`                       | `Vec<(String, String, String)>` | —        | unit |
| 84 | `execute.remove_command` | `ExecuteRpc` | `client.plugins().execute.remove_command(id)` | `(command_id)`             | `()`                            | —        | unit |
| 85 | `execute.save_command`   | `ExecuteRpc` | `client.plugins().execute.save_command(...)`  | `(command_id, event, cmd)` | `()`                            | —        | unit |

### extractor (2)

| #  | Spec method            | Trait          | Sub-client path                                 | Args       | Return            | Cassette | Test |
|----|------------------------|----------------|-------------------------------------------------|------------|-------------------|----------|------|
| 86 | `extractor.set_config` | `ExtractorRpc` | `client.plugins().extractor.set_config(config)` | `(config)` | `()`              | —        | unit |
| 87 | `extractor.get_config` | `ExtractorRpc` | `client.plugins().extractor.get_config()`       | `()`       | `ExtractorConfig` | —        | unit |

### label (8)

| #  | Spec method         | Trait      | Sub-client path                                | Args                       | Return         | Cassette | Test |
|----|---------------------|------------|------------------------------------------------|----------------------------|----------------|----------|------|
| 88 | `label.get_labels`  | `LabelRpc` | `client.plugins().label.get_labels()`          | `()`                       | `Vec<String>`  | —        | unit |
| 89 | `label.add`         | `LabelRpc` | `client.plugins().label.add(label_id)`         | `(label_id)`               | `()`           | —        | unit |
| 90 | `label.remove`      | `LabelRpc` | `client.plugins().label.remove(label_id)`      | `(label_id)`               | `()`           | —        | unit |
| 91 | `label.set_options` | `LabelRpc` | `client.plugins().label.set_options(...)`      | `(label_id, options_dict)` | `()`           | —        | unit |
| 92 | `label.get_options` | `LabelRpc` | `client.plugins().label.get_options(label_id)` | `(label_id)`               | `LabelOptions` | —        | unit |
| 93 | `label.set_torrent` | `LabelRpc` | `client.plugins().label.set_torrent(...)`      | `(torrent_id, label_id)`   | `()`           | —        | unit |
| 94 | `label.get_config`  | `LabelRpc` | `client.plugins().label.get_config()`          | `()`                       | `LabelConfig`  | —        | unit |
| 95 | `label.set_config`  | `LabelRpc` | `client.plugins().label.set_config(options)`   | `(options)`                | `()`           | —        | unit |

### notifications (3)

| #  | Spec method                        | Trait              | Sub-client path                                       | Args       | Return                  | Cassette | Test |
|----|------------------------------------|--------------------|-------------------------------------------------------|------------|-------------------------|----------|------|
| 96 | `notifications.set_config`         | `NotificationsRpc` | `client.plugins().notifications.set_config(config)`   | `(config)` | `()`                    | —        | unit |
| 97 | `notifications.get_config`         | `NotificationsRpc` | `client.plugins().notifications.get_config()`         | `()`       | `NotificationsConfig`   | —        | unit |
| 98 | `notifications.get_handled_events` | `NotificationsRpc` | `client.plugins().notifications.get_handled_events()` | `()`       | `Vec<(String, String)>` | —        | unit |

### scheduler (3)

| #   | Spec method            | Trait          | Sub-client path                                 | Args       | Return            | Cassette | Test |
|-----|------------------------|----------------|-------------------------------------------------|------------|-------------------|----------|------|
| 99  | `scheduler.set_config` | `SchedulerRpc` | `client.plugins().scheduler.set_config(config)` | `(config)` | `()`              | —        | unit |
| 100 | `scheduler.get_config` | `SchedulerRpc` | `client.plugins().scheduler.get_config()`       | `()`       | `SchedulerConfig` | —        | unit |
| 101 | `scheduler.get_state`  | `SchedulerRpc` | `client.plugins().scheduler.get_state()`        | `()`       | `String`          | —        | unit |

### stats (6)

| #   | Spec method                | Trait      | Sub-client path                               | Args               | Return                | Cassette | Test |
|-----|----------------------------|------------|-----------------------------------------------|--------------------|-----------------------|----------|------|
| 102 | `stats.get_stats`          | `StatsRpc` | `client.plugins().stats.get_stats(...)`       | `(keys, interval)` | `Option<StatsResult>` | —        | unit |
| 103 | `stats.get_totals`         | `StatsRpc` | `client.plugins().stats.get_totals()`         | `()`               | `StatsTotals`         | —        | unit |
| 104 | `stats.get_session_totals` | `StatsRpc` | `client.plugins().stats.get_session_totals()` | `()`               | `StatsTotals`         | —        | unit |
| 105 | `stats.set_config`         | `StatsRpc` | `client.plugins().stats.set_config(config)`   | `(config)`         | `()`                  | —        | unit |
| 106 | `stats.get_config`         | `StatsRpc` | `client.plugins().stats.get_config()`         | `()`               | `StatsConfig`         | —        | unit |
| 107 | `stats.get_intervals`      | `StatsRpc` | `client.plugins().stats.get_intervals()`      | `()`               | `Vec<i64>`            | —        | unit |

### toggle (2)

| #   | Spec method         | Trait       | Sub-client path                        | Args | Return | Cassette | Test |
|-----|---------------------|-------------|----------------------------------------|------|--------|----------|------|
| 108 | `toggle.get_status` | `ToggleRpc` | `client.plugins().toggle.get_status()` | `()` | `bool` | —        | unit |
| 109 | `toggle.toggle`     | `ToggleRpc` | `client.plugins().toggle.toggle()`     | `()` | `bool` | —        | unit |

### webui (3)

| #   | Spec method            | Trait      | Sub-client path                             | Args       | Return        | Cassette | Test |
|-----|------------------------|------------|---------------------------------------------|------------|---------------|----------|------|
| 110 | `webui.got_deluge_web` | `WebuiRpc` | `client.plugins().webui.got_deluge_web()`   | `()`       | `bool`        | —        | unit |
| 111 | `webui.set_config`     | `WebuiRpc` | `client.plugins().webui.set_config(config)` | `(config)` | `()`          | —        | unit |
| 112 | `webui.get_config`     | `WebuiRpc` | `client.plugins().webui.get_config()`       | `()`       | `WebuiConfig` | —        | unit |

## Summary

| Category      | Count   | Traits             | Implemented | Tested      |
|---------------|---------|--------------------|-------------|-------------|
| daemon        | 7       | `DaemonRpc`        | 7/7         | 7/7         |
| core torrents | 30      | `CoreTorrentRpc`   | 30/30       | 30/30       |
| core session  | 10      | `CoreSessionRpc`   | 10/10       | 10/10       |
| core config   | 5       | `CoreConfigRpc`    | 5/5         | 5/5         |
| core plugins  | 6       | `CorePluginsRpc`   | 6/6         | 6/6         |
| core accounts | 5       | `CoreAccountsRpc`  | 5/5         | 5/5         |
| core misc     | 3       | `CoreMiscRpc`      | 3/3         | 3/3         |
| core queue    | 1       | `CoreTorrentRpc`   | 1/1         | 1/1         |
| autoadd       | 10      | `AutoaddRpc`       | 10/10       | 10/10       |
| blocklist     | 4       | `BlocklistRpc`     | 4/4         | 4/4         |
| execute       | 4       | `ExecuteRpc`       | 4/4         | 4/4         |
| extractor     | 2       | `ExtractorRpc`     | 2/2         | 2/2         |
| label         | 8       | `LabelRpc`         | 8/8         | 8/8         |
| notifications | 3       | `NotificationsRpc` | 3/3         | 3/3         |
| scheduler     | 3       | `SchedulerRpc`     | 3/3         | 3/3         |
| stats         | 6       | `StatsRpc`         | 6/6         | 6/6         |
| toggle        | 2       | `ToggleRpc`        | 2/2         | 2/2         |
| webui         | 3       | `WebuiRpc`         | 3/3         | 3/3         |
| **Total**     | **112** | **18 traits**      | **112/112** | **112/112** |

## Cassette fixtures

| Fixture           | Methods covered                                                                             | Used by    |
|-------------------|---------------------------------------------------------------------------------------------|------------|
| `login_ok`        | (empty — login auto-served by mock)                                                          | unit + e2e |
| `login_bad`       | (empty — login auto-served by mock)                                                          | unit       |
| `free_space_low`  | `core.get_free_space` (5 GB)                                                                | unit + e2e |
| `free_space_high` | `core.get_free_space` (30 GB)                                                               | unit + e2e |
| `torrents_list`   | `core.get_torrents_status`                                                                   | unit + e2e |
| `remove_torrent`  | `core.get_torrents_status` + `core.remove_torrent`                                           | unit + e2e |
| `live-daemon`     | `daemon.info`, `daemon.get_version`, `core.get_free_space`, `core.get_torrents_status`, `core.get_session_status`, `core.get_config`, `core.get_enabled_plugins` | e2e (recorded from real daemon v2.1.2.dev0) |
| `torrent-lifecycle` | `core.get_torrent_status`, `core.get_magnet_uri`, `core.remove_torrent`                    | e2e (recorded from real daemon, Debian ISO add→verify→remove cycle) |

## Verification status

Methods verified against a live Deluge daemon (v2.1.2.dev0, libtorrent 2.0.10.0):

| Method                        | Verified | Notes                                                      |
|-------------------------------|----------|------------------------------------------------------------|
| `daemon.info`                 | ✅ live  | Returns `"2.1.2.dev0"`                                     |
| `daemon.login`                | ✅ live  | Auth level 10 (ADMIN)                                      |
| `daemon.get_version`          | ✅ live  | Returns `"2.1.2.dev0"`                                     |
| `daemon.get_method_list`      | ✅ live  | 79 methods (includes ltConfig plugin not in spec)          |
| `daemon.authorized_call`      | ✅ live  | Returns `true` for `core.remove_torrent`                   |
| `core.get_free_space`         | ✅ live  | Returns ~2.98 TB                                           |
| `core.get_torrents_status`    | ✅ live  | 2 torrents with full status dict                           |
| `core.get_torrent_status`     | ✅ live  | Single torrent status (Debian ISO)                         |
| `core.get_session_state`      | ✅ live  | 2 torrent hashes                                           |
| `core.get_session_status`     | ✅ live  | Full session metrics (12 typed + ~150 overflow keys)       |
| `core.get_config`             | ✅ live  | Full config dict (DaemonConfig deserializes with defaults) |
| `core.get_config_value`       | ✅ live  | Single key lookup (`download_location` → `/working`)       |
| `core.get_proxy`              | ✅ live  | ProxyConfig (type=0, no proxy)                             |
| `core.get_available_plugins`  | ✅ live  | 11 plugins                                                 |
| `core.get_enabled_plugins`    | ✅ live  | `["ltConfig"]`                                             |
| `core.get_external_ip`        | ✅ live  | `"146.70.117.77"`                                          |
| `core.get_libtorrent_version` | ✅ live  | `"2.0.10.0"`                                               |
| `core.get_listen_port`        | ✅ live  | `55337`                                                    |
| `core.is_session_paused`      | ✅ live  | `false`                                                    |
| `core.get_filter_tree`        | ✅ live  | Filter tree with state + owner fields                      |
| `core.get_known_accounts`     | ✅ live  | Account list                                               |
| `core.get_auth_levels_mappings` | ✅ live | Forward + reverse mappings                                 |
| `core.add_torrent_magnet`     | ✅ live  | Added Debian 12 ISO, returned torrent hash                 |
| `core.get_magnet_uri`         | ✅ live  | Reconstructed magnet from torrent hash                     |
| `core.remove_torrent`         | ✅ live  | Removed Debian ISO torrent, returned `true`                |
| `core.get_path_size`          | ✅ live  | Path size for `/working`                                   |
| `core.get_ssl_listen_port`    | ❌ N/A   | Method not available on this daemon version                |
| `label.get_labels`            | ❌ N/A   | Label plugin not enabled                                    |

All other methods are unit-tested only (not verified against a live daemon).

## Notes

- All 112 methods are fully implemented in the `deluge-rpc` crate with typed args and return values.
- All methods have unit tests in their respective `#[cfg(test)] mod tests` blocks.
- 26 methods verified against a live Deluge daemon (v2.1.2.dev0, libtorrent 2.0.10.0) — see "Verification status" above.
- 2 methods not available on the test daemon: `core.get_ssl_listen_port` (method not registered), `label.get_labels` (Label plugin not enabled).
- The mock server auto-serves `daemon.login` for any credentials — cassettes never contain login interactions or passwords.
- Cassettes are built programmatically in `deluge-retain/tests/common/cassettes.rs` and also available as JSON
  in `deluge-rpc-mock/fixtures/` (including 2 recorded from a live daemon).
- No gaps or TODOs remain.
