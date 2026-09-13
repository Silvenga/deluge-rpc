use crate::DelugeRpcError;
use crate::RencodeValue;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::{CompletionPaths, CreateTorrentRequest, CreateTorrentResult, GlobResult};
use crate::protocol::{DelugeRpcRequest, extract_single};
use crate::to_rencode_value;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Client for `core.*` misc RPC methods.
pub struct CoreMiscClient {
    dispatcher: DelugeClientDispatcher,
}

impl CoreMiscClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for CoreMiscClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

impl CoreMiscClient {
    /// Creates a torrent file from `request.path`. Returns the filename and base64-encoded filedump.
    pub async fn create_torrent(
        &self,
        request: &CreateTorrentRequest,
    ) -> Result<CreateTorrentResult, DelugeRpcError> {
        let result = self.dispatcher.dispatch(build_request(request)?).await?;
        let value = extract_single(&result)?;
        Ok(CreateTorrentResult::deserialize(&value)?)
    }
    /// Returns filesystem paths matching the glob pattern.
    pub async fn glob(&self, path: &str) -> Result<GlobResult, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.glob")
                    .with_args(vec![RencodeValue::Str(path.to_owned())]),
            )
            .await?;
        let value = extract_single(&result)?;
        Ok(GlobResult::deserialize(&value)?)
    }
    /// Returns path completions for a partial path input.
    pub async fn get_completion_paths(
        &self,
        completion_text: &str,
        show_hidden_files: bool,
    ) -> Result<CompletionPaths, DelugeRpcError> {
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
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.get_completion_paths").with_args(vec![args_value]),
            )
            .await?;
        let value = extract_single(&result)?;
        Ok(CompletionPaths::deserialize(&value)?)
    }
}

fn build_request(request: &CreateTorrentRequest) -> Result<DelugeRpcRequest, DelugeRpcError> {
    let kwargs = to_rencode_value(request)?;
    let kwargs = match kwargs {
        RencodeValue::Dict(map) => map,
        other => {
            return Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.create_torrent".into(),
                value: other,
            });
        }
    };

    Ok(DelugeRpcRequest::new("core.create_torrent")
        .with_args(vec![
            RencodeValue::Str(request.path.clone()),
            RencodeValue::Str(request.tracker.clone()),
            RencodeValue::Int(request.piece_length),
        ])
        .with_kwargs(kwargs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use crate::models::TorrentFormat;

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

    #[test]
    fn when_create_torrent_request_minimal_then_kwargs_empty() {
        let request =
            CreateTorrentRequest::new("/data/file.txt", "http://tracker/announce", 262144);

        let built = build_request(&request).expect("build request");

        assert_eq!(built.method, "core.create_torrent");
        assert_eq!(
            built.args,
            vec![
                RencodeValue::Str("/data/file.txt".into()),
                RencodeValue::Str("http://tracker/announce".into()),
                RencodeValue::Int(262144),
            ]
        );
        assert!(
            built.kwargs.is_empty(),
            "kwargs should be empty, got {:?}",
            built.kwargs
        );
    }

    #[test]
    fn when_create_torrent_request_full_then_kwargs_serialized() {
        let request =
            CreateTorrentRequest::new("/data/file.txt", "http://tracker/announce", 262144)
                .with_comment("a comment")
                .with_target("/out/file.torrent")
                .with_webseeds(vec!["http://seed/".into()])
                .with_private(true)
                .with_created_by("deluge-rpc")
                .with_trackers(vec![vec!["http://tier1/announce".into()]])
                .with_add_to_session(true)
                .with_torrent_format(TorrentFormat::Hybrid)
                .with_ca_cert("-----BEGIN CERTIFICATE-----");

        let built = build_request(&request).expect("build request");

        assert_eq!(
            built.kwargs.get(&RencodeValue::Str("comment".into())),
            Some(&RencodeValue::Str("a comment".into()))
        );
        assert_eq!(
            built.kwargs.get(&RencodeValue::Str("target".into())),
            Some(&RencodeValue::Str("/out/file.torrent".into()))
        );
        assert_eq!(
            built.kwargs.get(&RencodeValue::Str("webseeds".into())),
            Some(&RencodeValue::List(vec![RencodeValue::Str(
                "http://seed/".into()
            )]))
        );
        assert_eq!(
            built.kwargs.get(&RencodeValue::Str("private".into())),
            Some(&RencodeValue::Bool(true))
        );
        assert_eq!(
            built.kwargs.get(&RencodeValue::Str("created_by".into())),
            Some(&RencodeValue::Str("deluge-rpc".into()))
        );
        assert_eq!(
            built.kwargs.get(&RencodeValue::Str("trackers".into())),
            Some(&RencodeValue::List(vec![RencodeValue::List(vec![
                RencodeValue::Str("http://tier1/announce".into())
            ])]))
        );
        assert_eq!(
            built
                .kwargs
                .get(&RencodeValue::Str("add_to_session".into())),
            Some(&RencodeValue::Bool(true))
        );
        assert_eq!(
            built
                .kwargs
                .get(&RencodeValue::Str("torrent_format".into())),
            Some(&RencodeValue::Str("hybrid".into()))
        );
        assert_eq!(
            built.kwargs.get(&RencodeValue::Str("ca_cert".into())),
            Some(&RencodeValue::Str("-----BEGIN CERTIFICATE-----".into()))
        );
    }
}
