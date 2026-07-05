use crate::client::caller::RpcCaller;
use crate::models::{CompletionPaths, CreateTorrentResult, GlobResult};
use crate::protocol::{extract_single, DelugeRpcRequest};
use crate::rencode::RencodeValue;
use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::BTreeMap;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait CoreMiscRpc: Send + Sync {
    #[expect(
        clippy::too_many_arguments,
        reason = "create_torrent has many optional params per Deluge API"
    )]
    async fn create_torrent(
        &self,
        path: &str,
        tracker: &str,
        piece_length: i64,
        comment: Option<String>,
        target: Option<String>,
        web_seeds: Option<Vec<String>>,
        private: bool,
        created_by: Option<String>,
        trackers: Option<Vec<Vec<String>>>,
        add_to_session: bool,
    ) -> anyhow::Result<CreateTorrentResult>;
    async fn glob(&self, path: &str) -> anyhow::Result<GlobResult>;
    async fn get_completion_paths(
        &self,
        completion_text: &str,
        show_hidden_files: bool,
    ) -> anyhow::Result<CompletionPaths>;
}

pub struct CoreMiscClient {
    caller: RpcCaller,
}

impl CoreMiscClient {
    pub(crate) fn new(caller: RpcCaller) -> Self {
        Self { caller }
    }
}

impl Clone for CoreMiscClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl CoreMiscRpc for CoreMiscClient {
    async fn create_torrent(
        &self,
        path: &str,
        tracker: &str,
        piece_length: i64,
        comment: Option<String>,
        target: Option<String>,
        web_seeds: Option<Vec<String>>,
        private: bool,
        created_by: Option<String>,
        trackers: Option<Vec<Vec<String>>>,
        add_to_session: bool,
    ) -> anyhow::Result<CreateTorrentResult> {
        let mut kwargs = BTreeMap::new();
        if let Some(c) = comment {
            kwargs.insert(RencodeValue::Str("comment".into()), RencodeValue::Str(c));
        }
        if let Some(t) = target {
            kwargs.insert(RencodeValue::Str("target".into()), RencodeValue::Str(t));
        }
        if let Some(ws) = web_seeds {
            let ws_values: Vec<RencodeValue> = ws.into_iter().map(RencodeValue::Str).collect();
            kwargs.insert(
                RencodeValue::Str("webseeds".into()),
                RencodeValue::List(ws_values),
            );
        }
        kwargs.insert(
            RencodeValue::Str("private".into()),
            RencodeValue::Bool(private),
        );
        if let Some(cb) = created_by {
            kwargs.insert(
                RencodeValue::Str("created_by".into()),
                RencodeValue::Str(cb),
            );
        }
        if let Some(tr) = trackers {
            let tr_values: Vec<RencodeValue> = tr
                .into_iter()
                .map(|tier| RencodeValue::List(tier.into_iter().map(RencodeValue::Str).collect()))
                .collect();
            kwargs.insert(
                RencodeValue::Str("trackers".into()),
                RencodeValue::List(tr_values),
            );
        }
        kwargs.insert(
            RencodeValue::Str("add_to_session".into()),
            RencodeValue::Bool(add_to_session),
        );

        let result = self
            .caller
            .dispatch(
                DelugeRpcRequest::new("core.create_torrent")
                    .with_args(vec![
                        RencodeValue::Str(path.to_owned()),
                        RencodeValue::Str(tracker.to_owned()),
                        RencodeValue::Int(piece_length),
                    ])
                    .with_kwargs(kwargs),
            )
            .await
            .context("core.create_torrent RPC failed")?;
        let value = extract_single(&result)?;
        CreateTorrentResult::deserialize(&value).context("deserializing create torrent result")
    }

    async fn glob(&self, path: &str) -> anyhow::Result<GlobResult> {
        let result = self
            .caller
            .dispatch(
                DelugeRpcRequest::new("core.glob")
                    .with_args(vec![RencodeValue::Str(path.to_owned())]),
            )
            .await
            .context("core.glob RPC failed")?;
        let value = extract_single(&result)?;
        GlobResult::deserialize(&value).context("deserializing glob result")
    }

    async fn get_completion_paths(
        &self,
        completion_text: &str,
        show_hidden_files: bool,
    ) -> anyhow::Result<CompletionPaths> {
        let mut args_map = BTreeMap::new();
        args_map.insert(
            RencodeValue::Str("completion_text".into()),
            RencodeValue::Str(completion_text.to_owned()),
        );
        args_map.insert(
            RencodeValue::Str("show_hidden_files".into()),
            RencodeValue::Bool(show_hidden_files),
        );
        let args_value = RencodeValue::Dict(args_map);

        let result = self
            .caller
            .dispatch(
                DelugeRpcRequest::new("core.get_completion_paths").with_args(vec![args_value]),
            )
            .await
            .context("core.get_completion_paths RPC failed")?;
        let value = extract_single(&result)?;
        CompletionPaths::deserialize(&value).context("deserializing completion paths")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;

    #[test]
    fn when_core_glob_then_vec_string() {
        let response = RencodeValue::List(vec![
            RencodeValue::Str("/downloads/file1.mkv".into()),
            RencodeValue::Str("/downloads/file2.mkv".into()),
        ]);
        let value = extract_single(&response).expect("extract");
        let result: GlobResult = GlobResult::deserialize(&value).expect("deserialize");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "/downloads/file1.mkv");
    }

    #[test]
    fn when_core_create_torrent_then_result() {
        let response = RencodeValue::List(vec![
            RencodeValue::Str("my.torrent".into()),
            RencodeValue::Str("base64data".into()),
        ]);
        let value = extract_single(&response).expect("extract");
        let result: CreateTorrentResult =
            CreateTorrentResult::deserialize(&value).expect("deserialize");
        assert_eq!(result.filename, "my.torrent");
        assert_eq!(result.file_dump, "base64data");
    }
}
