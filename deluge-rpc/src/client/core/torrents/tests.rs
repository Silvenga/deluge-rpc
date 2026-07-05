use super::*;
use crate::models::torrents::{
    AddTorrentFileResult, AddTorrentFilesResult, FilterDict, FilterTree, PrefetchMagnetResult,
    RemoveTorrentsResult, TorrentEntry, TorrentStatus,
};
use crate::protocol::{extract_single, extract_single_dict, extract_single_int};
use crate::rencode::RencodeValue;
use mockall::predicate;
use std::collections::BTreeMap;

// --- add_torrent_file / add_torrent_file_async / add_torrent_url ---

#[test]
fn when_add_torrent_file_response_str_then_some() {
    let response = RencodeValue::Str(
        "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into(),
    );
    let value = extract_single(&response).expect("extract");
    let result: AddTorrentFileResult = match value {
        RencodeValue::Str(s) => Some(s),
        RencodeValue::None => None,
        _ => panic!("unexpected value: {value:?}"),
    };
    assert_eq!(
        result,
        Some("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into())
    );
}

#[test]
fn when_add_torrent_file_response_none_then_none() {
    let response = RencodeValue::None;
    let value = extract_single(&response).expect("extract");
    let result: AddTorrentFileResult = match value {
        RencodeValue::Str(s) => Some(s),
        RencodeValue::None => None,
        _ => panic!("unexpected value: {value:?}"),
    };
    assert_eq!(result, None);
}

// --- add_torrent_files ---

#[test]
fn when_add_torrent_files_response_empty_then_all_succeeded() {
    let response = RencodeValue::List(vec![]);
    let value = extract_single(&response).expect("extract");
    let result: AddTorrentFilesResult =
        AddTorrentFilesResult::deserialize(&value).expect("deserialize");
    assert!(result.is_empty());
}

#[test]
fn when_add_torrent_files_response_errors_then_deserialized() {
    let response = RencodeValue::List(vec![RencodeValue::Str(
        "failed to add torrent".into(),
    )]);
    let value = extract_single(&response).expect("extract");
    let result: AddTorrentFilesResult =
        AddTorrentFilesResult::deserialize(&value).expect("deserialize");
    assert_eq!(result, vec!["failed to add torrent"]);
}

// --- add_torrent_magnet ---

#[test]
fn when_add_torrent_magnet_response_str_then_ok() {
    let response = RencodeValue::Str(
        "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into(),
    );
    let value = extract_single(&response).expect("extract");
    match value {
        RencodeValue::Str(s) => assert_eq!(s, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111"),
        other => panic!("expected str, got {other:?}"),
    }
}

// --- prefetch_magnet_metadata ---

#[test]
fn when_prefetch_magnet_metadata_response_tuple_then_deserialized() {
    let response = RencodeValue::List(vec![
        RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
        RencodeValue::Bytes(b"bencoded-data".to_vec()),
    ]);
    let value = extract_single(&response).expect("extract");
    let result: PrefetchMagnetResult =
        PrefetchMagnetResult::deserialize(&value).expect("deserialize");
    assert_eq!(
        result.torrent_id,
        "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111"
    );
    assert_eq!(result.metadata, b"bencoded-data");
}

// --- remove_torrent ---

#[test]
fn when_remove_torrent_response_true_then_bool() {
    let response = RencodeValue::Bool(true);
    let value = extract_single(&response).expect("extract");
    match value {
        RencodeValue::Bool(b) => assert!(b),
        other => panic!("expected bool, got {other:?}"),
    }
}

#[test]
fn when_remove_torrent_response_false_then_bool() {
    let response = RencodeValue::Bool(false);
    let value = extract_single(&response).expect("extract");
    match value {
        RencodeValue::Bool(b) => assert!(!b),
        other => panic!("expected bool, got {other:?}"),
    }
}

// --- remove_torrents ---

#[test]
fn when_remove_torrents_response_empty_then_all_succeeded() {
    let response = RencodeValue::List(vec![]);
    let value = extract_single(&response).expect("extract");
    let result: RemoveTorrentsResult =
        RemoveTorrentsResult::deserialize(&value).expect("deserialize");
    assert!(result.is_empty());
}

#[test]
fn when_remove_torrents_response_errors_then_deserialized() {
    let response = RencodeValue::List(vec![RencodeValue::List(vec![
        RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
        RencodeValue::Str("torrent not found".into()),
    ])]);
    let value = extract_single(&response).expect("extract");
    let result: RemoveTorrentsResult =
        RemoveTorrentsResult::deserialize(&value).expect("deserialize");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
    assert_eq!(result[0].1, "torrent not found");
}

// --- get_torrent_status ---

#[test]
fn when_get_torrent_status_response_dict_then_torrent_status() {
    let mut map = BTreeMap::new();
    map.insert(
        RencodeValue::Str("name".into()),
        RencodeValue::Str("test-torrent".into()),
    );
    map.insert(
        RencodeValue::Str("state".into()),
        RencodeValue::Str("Downloading".into()),
    );
    map.insert(
        RencodeValue::Str("progress".into()),
        RencodeValue::Float(50.0),
    );
    map.insert(
        RencodeValue::Str("hash".into()),
        RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
    );
    map.insert(RencodeValue::Str("ratio".into()), RencodeValue::Float(-1.0));
    map.insert(RencodeValue::Str("eta".into()), RencodeValue::Int(-1));
    map.insert(
        RencodeValue::Str("completed_time".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("last_seen_complete".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("time_since_download".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("time_since_transfer".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("time_since_upload".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("seeds_peers_ratio".into()),
        RencodeValue::Float(-1.0),
    );
    map.insert(
        RencodeValue::Str("max_connections".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("max_download_speed".into()),
        RencodeValue::Float(-1.0),
    );
    map.insert(
        RencodeValue::Str("max_upload_slots".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("max_upload_speed".into()),
        RencodeValue::Float(-1.0),
    );
    let response = RencodeValue::Dict(map);

    let value = extract_single(&response).expect("extract");
    let status: TorrentStatus = TorrentStatus::deserialize(&value).expect("deserialize");

    assert_eq!(status.name, "test-torrent");
    assert_eq!(status.state, "Downloading");
    assert!((status.progress - 50.0).abs() < f64::EPSILON);
    assert_eq!(status.ratio, None);
    assert_eq!(status.eta, None);
}

// --- get_torrents_status ---

fn make_torrent_entry_dict(name: &str, hash: &str) -> BTreeMap<RencodeValue, RencodeValue> {
    let mut map = BTreeMap::new();
    map.insert(
        RencodeValue::Str("name".into()),
        RencodeValue::Str(name.into()),
    );
    map.insert(
        RencodeValue::Str("state".into()),
        RencodeValue::Str("Seeding".into()),
    );
    map.insert(
        RencodeValue::Str("progress".into()),
        RencodeValue::Float(100.0),
    );
    map.insert(RencodeValue::Str("ratio".into()), RencodeValue::Float(2.5));
    map.insert(
        RencodeValue::Str("hash".into()),
        RencodeValue::Str(hash.into()),
    );
    map.insert(RencodeValue::Str("eta".into()), RencodeValue::Int(-1));
    map.insert(
        RencodeValue::Str("completed_time".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("last_seen_complete".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("time_since_download".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("time_since_transfer".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("time_since_upload".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("seeds_peers_ratio".into()),
        RencodeValue::Float(-1.0),
    );
    map.insert(
        RencodeValue::Str("max_connections".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("max_download_speed".into()),
        RencodeValue::Float(-1.0),
    );
    map.insert(
        RencodeValue::Str("max_upload_slots".into()),
        RencodeValue::Int(-1),
    );
    map.insert(
        RencodeValue::Str("max_upload_speed".into()),
        RencodeValue::Float(-1.0),
    );
    map
}

#[test]
fn when_get_torrents_status_response_dict_then_vec_torrent_entry() {
    let mut result_dict = BTreeMap::new();
    result_dict.insert(
        RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
        RencodeValue::Dict(make_torrent_entry_dict(
            "torrent-a",
            "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111",
        )),
    );
    result_dict.insert(
        RencodeValue::Str("bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222".into()),
        RencodeValue::Dict(make_torrent_entry_dict(
            "torrent-b",
            "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222",
        )),
    );
    let response = RencodeValue::Dict(result_dict);

    let result_dict = extract_single_dict(&response, "core.get_torrents_status").expect("extract");

    let mut entries: Vec<(String, &RencodeValue)> = result_dict
        .iter()
        .filter_map(|(k, v)| match (k, v) {
            (RencodeValue::Str(id), fields) => Some((id.clone(), fields)),
            _ => None,
        })
        .collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut out = Vec::with_capacity(entries.len());
    for (info_hash, fields) in entries {
        let mut entry =
            TorrentEntry::deserialize(fields).unwrap_or_else(|_| panic!("deserialize {info_hash}"));
        entry.info_hash = info_hash;
        out.push(entry);
    }

    assert_eq!(out.len(), 2);
    assert_eq!(out[0].info_hash, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
    assert_eq!(out[0].status.name, "torrent-a");
    assert_eq!(out[1].info_hash, "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222");
    assert_eq!(out[1].status.name, "torrent-b");
}

#[test]
fn when_get_torrents_status_response_empty_dict_then_empty_vec() {
    let response = RencodeValue::Dict(BTreeMap::new());

    let result_dict = extract_single_dict(&response, "core.get_torrents_status").expect("extract");
    assert!(result_dict.is_empty());
}

// --- get_filter_tree ---

#[test]
fn when_get_filter_tree_response_dict_then_filter_tree() {
    let state_entries = vec![
        RencodeValue::List(vec![RencodeValue::Str("All".into()), RencodeValue::Int(42)]),
        RencodeValue::List(vec![
            RencodeValue::Str("Seeding".into()),
            RencodeValue::Int(10),
        ]),
    ];

    let mut filter_dict = BTreeMap::new();
    filter_dict.insert(
        RencodeValue::Str("state".into()),
        RencodeValue::List(state_entries),
    );

    let response = RencodeValue::Dict(filter_dict);

    let value = extract_single(&response).expect("extract");
    let tree: FilterTree = FilterTree::deserialize(&value).expect("deserialize");

    let state = tree.get("state").expect("state key");
    assert_eq!(state.len(), 2);
    assert_eq!(state[0].value, "All");
    assert_eq!(state[0].count, 42);
    assert_eq!(state[1].value, "Seeding");
    assert_eq!(state[1].count, 10);
}

// --- get_session_state ---

#[test]
fn when_get_session_state_response_list_then_vec_string() {
    let response = RencodeValue::List(vec![
        RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
        RencodeValue::Str("bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222".into()),
    ]);
    let value = extract_single(&response).expect("extract");
    match value {
        RencodeValue::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    RencodeValue::Str(s) => out.push(s),
                    other => panic!("expected str, got {other:?}"),
                }
            }
            assert_eq!(out.len(), 2);
            assert_eq!(out[0], "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
            assert_eq!(out[1], "bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222");
        }
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn when_get_session_state_response_empty_list_then_empty_vec() {
    let response = RencodeValue::List(vec![]);
    let value = extract_single(&response).expect("extract");
    match value {
        RencodeValue::List(items) => assert!(items.is_empty()),
        other => panic!("expected list, got {other:?}"),
    }
}

// --- get_magnet_uri ---

#[test]
fn when_get_magnet_uri_response_str_then_string() {
    let response = RencodeValue::Str(
        "magnet:?xt=urn:btih:aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into(),
    );
    let value = extract_single(&response).expect("extract");
    match value {
        RencodeValue::Str(s) => {
            assert_eq!(
                s,
                "magnet:?xt=urn:btih:aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111"
            );
        }
        other => panic!("expected str, got {other:?}"),
    }
}

// --- get_path_size ---

#[test]
fn when_get_path_size_response_int_then_i64() {
    let response = RencodeValue::Int(1_073_741_824);
    let bytes = extract_single_int(&response, "core.get_path_size").expect("extract");
    assert_eq!(bytes, 1_073_741_824);
}

// --- MockCoreTorrentRpc ---

#[tokio::test]
async fn when_mock_core_torrent_rpc_then_expectations_met() {
    let mut mock = MockCoreTorrentRpc::new();

    mock.expect_remove_torrent()
        .with(predicate::eq("hash1"), predicate::eq(true))
        .returning(|_, _| Ok(true));

    mock.expect_get_torrents_status()
        .returning(|_, _, _| Ok(vec![]));

    mock.expect_get_session_state().returning(|| Ok(vec![]));

    let removed = mock.remove_torrent("hash1", true).await.expect("remove");
    assert!(removed);

    let entries = mock
        .get_torrents_status(&FilterDict::default(), &[], false)
        .await
        .expect("get_torrents_status");
    assert!(entries.is_empty());

    let state = mock.get_session_state().await.expect("get_session_state");
    assert!(state.is_empty());
}
