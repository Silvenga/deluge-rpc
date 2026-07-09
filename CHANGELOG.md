# Changelog

## [0.3.0](https://github.com/Silvenga/deluge-rpc/compare/v0.2.0...v0.3.0) (2026-07-09)


### Features

* **cli:** add status command for operator overview ([#8](https://github.com/Silvenga/deluge-rpc/issues/8)) ([2735f37](https://github.com/Silvenga/deluge-rpc/commit/2735f37b69cef12d4b3d78b6a1e77f3efd9ed83b))

## [0.2.0](https://github.com/Silvenga/deluge-rpc/compare/v0.1.2...v0.2.0) (2026-07-09)


### Features

* added support for event subscriptions ([ed8e27c](https://github.com/Silvenga/deluge-rpc/commit/ed8e27c18d4fdcdce89968dd0e5c49d3f42cbbf5))

## [0.1.2](https://github.com/Silvenga/deluge-rpc/compare/v0.1.1...v0.1.2) (2026-07-09)


### Bug Fixes

* **ci:** remove release-type input so release-please reads config-file ([#5](https://github.com/Silvenga/deluge-rpc/issues/5)) ([8515d9e](https://github.com/Silvenga/deluge-rpc/commit/8515d9ef907e80a8415fb939bbbcc4d8792202f8))

## [0.1.1](https://github.com/Silvenga/deluge-rpc/compare/v0.1.0...v0.1.1) (2026-07-09)


### Bug Fixes

* **docs:** fixed visablity of public api] ([c5fb079](https://github.com/Silvenga/deluge-rpc/commit/c5fb079b26a21550ffc58be4739c63a67fd4813a))
* removed internal symboles from public api ([ee55404](https://github.com/Silvenga/deluge-rpc/commit/ee55404d5fbb1948093c58b7e0734784b36adea0))
* removed traits from public api ([f784427](https://github.com/Silvenga/deluge-rpc/commit/f784427e7eb281bb28753b9ca5ea0e0d8f5daaf3))

## 0.1.0 (2026-07-09)


### ⚠ BREAKING CHANGES

* replace wiremock HTTP mocks with MockDelugeDaemon + trait mocks
* **client:** replaces Web UI JSON-RPC (HTTP /json, reqwest, cookie auth) with native daemon RPC (TCP+TLS, rencode wire format, daemon.login auth). DelugeClient now holds host/port/username/password + TLS transport instead of reqwest client + URL. RPC calls use rencode encoding with 5-byte header + zlib framing. RPC_EVENT messages are logged and discarded. 17 unit tests for request encoding, response parsing, error handling, and event interleaving. Placeholder submodule stubs deleted. Integration tests
* **config:** the url field is removed from [[hosts]] config. Use host, port (default 58846), and username (default localclient) instead. The password/password_env fields are unchanged. Existing configs must be rewritten. Temporary shim in process_host constructs an HTTP URL until the daemon RPC client is implemented in task 3.

### Features

* build out CLI subcommands and add e2e cassette tests for all 112 RPC methods ([#2](https://github.com/Silvenga/deluge-rpc/issues/2)) ([9e76a3c](https://github.com/Silvenga/deluge-rpc/commit/9e76a3c245dd1c38fbb0bd7ecdd5b58a0514c6d7))
* **client:** rewrite DelugeClient for native daemon RPC protocol ([a771bc0](https://github.com/Silvenga/deluge-rpc/commit/a771bc0170fa1e4cf93f83f8751d5f8309028b03))
* **config:** migrate host config from Web UI url to daemon host/port/username ([d161bb2](https://github.com/Silvenga/deluge-rpc/commit/d161bb263a04a97b850acba43b611fbc0e61e03d))
* **deluge-retain:** implement automatic torrent retention for Deluge ([a93a058](https://github.com/Silvenga/deluge-rpc/commit/a93a0585b2aea4cad1a3ab33975bfcde53f764ce))
* **deluge-rpc-mock, deluge-cli:** add mock replay server and CLI crates ([326310e](https://github.com/Silvenga/deluge-rpc/commit/326310eccb8e6b16bcae6dc01a2e74675a0a904b))
* **deluge-rpc-mock, deluge-cli:** auto-serve login, append-mode recorder, dev comments ([9a674e2](https://github.com/Silvenga/deluge-rpc/commit/9a674e2671b05755fa871a65ad0d8fa8829fbf41))
* **deluge-rpc/client:** add daemon, core, and plugin domain traits with sub-clients ([9759485](https://github.com/Silvenga/deluge-rpc/commit/97594853165b696e3b3758e4cd858644a80102fa))
* **deluge-rpc/client:** add DelugeClient with lazy reconnect and sub-client properties ([63bcb24](https://github.com/Silvenga/deluge-rpc/commit/63bcb24319d40e796215fcdd12269151b2671f1e))
* **deluge-rpc/models:** add sentinel deserialize_with helpers and dict-key extraction ([8d23a8f](https://github.com/Silvenga/deluge-rpc/commit/8d23a8f8a9b4fc15a9476167e473b61f2c16c1e6))
* **deluge-rpc/models:** add torrent, session/config, and plugin domain models ([38bac10](https://github.com/Silvenga/deluge-rpc/commit/38bac1022c57a7228fbf6042c1ce74cfe1df57e0))
* **deluge-rpc/rencode:** add serde Deserializer for RencodeValue and tagged JSON conversion ([a50eac2](https://github.com/Silvenga/deluge-rpc/commit/a50eac257b92b188b12cd0a5e963a7552f261e74))
* **rencode:** add clean-room rencode codec for Deluge daemon RPC ([c47123d](https://github.com/Silvenga/deluge-rpc/commit/c47123db9e44a4575f303de24c6da3bd573ca762))
* **transport:** add TLS transport with zlib framing for daemon RPC ([430ab9e](https://github.com/Silvenga/deluge-rpc/commit/430ab9ef791f8d6717c4081fd574bff0ac48d685))


### Bug Fixes

* address all code review findings (3 high, 5 medium, 3 low) ([5e3d6a8](https://github.com/Silvenga/deluge-rpc/commit/5e3d6a841dfd879147447a82db739824da6b9c95))
* DaemonConfig serde default, plain JSON call I/O, SessionStatus plain JSON output ([4495cd4](https://github.com/Silvenga/deluge-rpc/commit/4495cd44d7b3e19133dcc022401f99bc45f48b44))
* **deluge-rpc/client:** detect dead connections during RPC recv loop ([b2ced43](https://github.com/Silvenga/deluge-rpc/commit/b2ced43484be655024c0d372083f5e4a7d65f74e))
* **deluge-rpc/protocol:** default args to empty list, not [None] ([bb25aae](https://github.com/Silvenga/deluge-rpc/commit/bb25aaed3af52950d53b80ed7b9e77bcb7faf0da))
* **deluge-rpc:** extract RpcCaller from mod.rs and remove #[allow] lints ([9688625](https://github.com/Silvenga/deluge-rpc/commit/9688625a4b07af2111f8b1ffd50f1e8c5ef55b9b))
* **deluge-rpc:** strip backward-compat from protocol decoder, use real bare format only ([7492282](https://github.com/Silvenga/deluge-rpc/commit/7492282e705d78a5c4493bff50757ff8a7592d6f))


### Miscellaneous Chores

* release 0.1.0 ([ff2f9d3](https://github.com/Silvenga/deluge-rpc/commit/ff2f9d35aba3a6033d865d860ada26938aaa45c9))


### Tests

* replace wiremock HTTP mocks with MockDelugeDaemon + trait mocks ([555d79b](https://github.com/Silvenga/deluge-rpc/commit/555d79b01713092d12798f342e1698af8f787d2b))
