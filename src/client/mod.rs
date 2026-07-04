//! Deluge Web JSON-RPC client.
//!
//! All methods live on [`DelugeClient`] in this module. The submodule
//! files (`auth`, `free_space`, `remove`, `update_ui`) are kept as
//! empty placeholders for future per-method documentation.

mod auth;
mod free_space;
mod remove;
mod update_ui;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, anyhow};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::torrent::TorrentInfo;

/// JSON-RPC response envelope returned by the Deluge Web UI.
#[derive(Debug, serde::Deserialize)]
struct RpcResponse {
    result: Option<Value>,
    error: Option<String>,
}

/// Wrapper for the `result` object of `web.update_ui`, whose `torrents`
/// field is the map keyed by info hash.
#[derive(Debug, serde::Deserialize)]
struct UpdateUiResult {
    torrents: HashMap<String, Value>,
}

/// Thin async client for the Deluge Web JSON-RPC API.
///
/// Holds a cookie-enabled [`reqwest::Client`] so that the session
/// cookie set by `auth.login` is replayed on subsequent calls.
pub struct DelugeClient {
    url: String,
    password: String,
    client: reqwest::Client,
    request_id: AtomicU32,
}

impl DelugeClient {
    /// Create a new client targeting `url` (the `/json` endpoint) that
    /// authenticates with `password`.
    pub fn new(url: String, password: String) -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            url,
            password,
            client,
            request_id: AtomicU32::new(1),
        }
    }

    fn next_id(&self) -> u32 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Issue a JSON-RPC call and return the deserialized `result`.
    ///
    /// Returns an error if the HTTP request fails, the response body
    /// is not valid JSON-RPC, or the server reports an `error`.
    async fn rpc_call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> anyhow::Result<T> {
        let id = self.next_id();
        let body = json!({
            "method": method,
            "params": params,
            "id": id,
        });

        let resp = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("HTTP request for Deluge `{method}` failed"))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Deluge API `{method}` failed: {status} {text}"));
        }

        let envelope: RpcResponse = resp
            .json()
            .await
            .with_context(|| format!("Deluge API `{method}` returned non-JSON-RPC body"))?;

        if let Some(err) = envelope.error {
            return Err(anyhow!("Deluge API `{method}` error: {err}"));
        }

        let result = envelope
            .result
            .ok_or_else(|| anyhow!("Deluge API `{method}` returned null result"))?;

        serde_json::from_value::<T>(result)
            .with_context(|| format!("Deluge API `{method}` result did not match expected type"))
    }

    /// Authenticate against the daemon. The session cookie is stored
    /// automatically by the cookie jar.
    pub async fn login(&self) -> anyhow::Result<()> {
        let ok: bool = self.rpc_call("auth.login", json!([self.password])).await?;
        if !ok {
            return Err(anyhow!("Deluge auth.login returned false (bad password?)"));
        }
        Ok(())
    }

    /// Query free disk space in bytes at the daemon's default download
    /// location.
    pub async fn get_free_space(&self) -> anyhow::Result<u64> {
        self.rpc_call("core.get_free_space", json!([Value::Null]))
            .await
    }

    /// Fetch the current status of all torrents.
    ///
    /// Requests the explicit field set required to populate
    /// [`TorrentInfo`]; the daemon returns a `result` object whose
    /// `torrents` field is a map keyed by info hash, which is flattened
    /// into a [`Vec<TorrentInfo>`].
    pub async fn get_torrents(&self) -> anyhow::Result<Vec<TorrentInfo>> {
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
        ];

        let raw: UpdateUiResult = self.rpc_call("web.update_ui", json!([keys, {}])).await?;

        let mut pairs: Vec<(String, Value)> = raw.torrents.into_iter().collect();
        pairs.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut out = Vec::with_capacity(pairs.len());
        for (info_hash, status) in pairs {
            let mut info: TorrentInfo = serde_json::from_value(status)
                .with_context(|| format!("parsing torrent `{info_hash}`"))?;
            info.info_hash = info_hash;
            out.push(info);
        }
        Ok(out)
    }

    /// Remove a torrent by info hash. `remove_data = true` deletes the
    /// downloaded files as well.
    pub async fn remove_torrent(&self, id: &str) -> anyhow::Result<bool> {
        self.rpc_call("core.remove_torrent", json!([id, true]))
            .await
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::str_to_string,
    reason = "test assertions use expect for clarity and string literals use to_string for ergonomics"
)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn spawn_server() -> MockServer {
        MockServer::start().await
    }

    #[tokio::test]
    async fn when_login_succeeds_then_returns_ok() {
        let server = spawn_server().await;
        Mock::given(method("POST"))
            .and(path("/json"))
            .and(header("content-type", "application/json"))
            .and(body_partial_json(json!({
                "method": "auth.login",
                "params": ["secret"],
                "id": 1,
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": true,
                "error": null,
                "id": 1,
            })))
            .mount(&server)
            .await;

        let client = DelugeClient::new(format!("{}/json", server.uri()), "secret".to_string());
        client.login().await.expect("login should succeed");
    }

    #[tokio::test]
    async fn when_login_returns_false_then_returns_error() {
        let server = spawn_server().await;
        Mock::given(method("POST"))
            .and(path("/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": false,
                "error": null,
                "id": 1,
            })))
            .mount(&server)
            .await;

        let client = DelugeClient::new(format!("{}/json", server.uri()), "wrong".to_string());
        let err = client.login().await.expect_err("should fail");
        assert!(
            err.to_string().contains("bad password"),
            "error should mention bad password, got: {err}"
        );
    }

    #[tokio::test]
    async fn when_login_error_field_set_then_returns_error() {
        let server = spawn_server().await;
        Mock::given(method("POST"))
            .and(path("/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": null,
                "error": "bad password",
                "id": 1,
            })))
            .mount(&server)
            .await;

        let client = DelugeClient::new(format!("{}/json", server.uri()), "x".to_string());
        let err = client.login().await.expect_err("should fail");
        assert!(
            err.to_string().contains("bad password"),
            "error should propagate server message, got: {err}"
        );
    }

    #[tokio::test]
    async fn when_get_free_space_then_returns_bytes() {
        let server = spawn_server().await;
        Mock::given(method("POST"))
            .and(path("/json"))
            .and(body_partial_json(json!({
                "method": "core.get_free_space",
                "params": [Value::Null],
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": 1_073_741_824_u64,
                "error": null,
                "id": 1,
            })))
            .mount(&server)
            .await;

        let client = DelugeClient::new(format!("{}/json", server.uri()), "p".to_string());
        let bytes = client.get_free_space().await.expect("should parse");
        assert_eq!(bytes, 1_073_741_824);
    }

    #[tokio::test]
    async fn when_get_torrents_then_parses_into_vec() {
        let server = spawn_server().await;
        let torrents = json!({
            "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111": {
                "name": "torrent-one",
                "state": "Seeding",
                "progress": 100.0,
                "ratio": 2.5,
                "total_seeds": 10,
                "num_seeds": 5,
                "time_added": 1_700_000_000.0_f64,
                "total_done": 1_048_576_u64,
                "total_uploaded": 2_097_152_u64,
                "is_finished": true,
                "download_location": "/data"
            },
            "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222": {
                "name": "torrent-two",
                "state": "Downloading",
                "progress": 42.0,
                "ratio": 0.1,
                "total_seeds": 20,
                "num_seeds": 3,
                "time_added": 1_700_000_100.0_f64,
                "total_done": 524_288_u64,
                "total_uploaded": 1024_u64,
                "is_finished": false,
                "download_location": "/data"
            }
        });

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
            .mount(&server)
            .await;

        let client = DelugeClient::new(format!("{}/json", server.uri()), "p".to_string());
        let list = client.get_torrents().await.expect("should parse");
        assert_eq!(list.len(), 2);
        let by_hash: HashMap<String, TorrentInfo> =
            list.into_iter().map(|t| (t.info_hash.clone(), t)).collect();
        let one = by_hash
            .get("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111")
            .expect("torrent one present");
        assert_eq!(one.name, "torrent-one");
        assert_eq!(one.state, "Seeding");
        assert!((one.progress - 100.0).abs() < f64::EPSILON);
        assert!(one.is_finished);
    }

    #[tokio::test]
    async fn when_remove_torrent_then_returns_true() {
        let server = spawn_server().await;
        Mock::given(method("POST"))
            .and(path("/json"))
            .and(body_partial_json(json!({
                "method": "core.remove_torrent",
                "params": ["deadbeef", true],
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": true,
                "error": null,
                "id": 1,
            })))
            .mount(&server)
            .await;

        let client = DelugeClient::new(format!("{}/json", server.uri()), "p".to_string());
        let ok = client
            .remove_torrent("deadbeef")
            .await
            .expect("should parse");
        assert!(ok);
    }

    #[tokio::test]
    async fn when_http_error_then_returns_error() {
        let server = spawn_server().await;
        Mock::given(method("POST"))
            .and(path("/json"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .mount(&server)
            .await;

        let client = DelugeClient::new(format!("{}/json", server.uri()), "p".to_string());
        let err = client.get_free_space().await.expect_err("should fail");
        assert!(
            err.to_string().contains("500"),
            "error should include status, got: {err}"
        );
    }
}
