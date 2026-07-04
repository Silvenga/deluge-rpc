use crate::rencode::RencodeValue;
use crate::torrent::TorrentInfo;
use anyhow::anyhow;
use std::collections::BTreeMap;

pub fn parse_torrent(
    info_hash: &str,
    fields: &BTreeMap<RencodeValue, RencodeValue>,
) -> anyhow::Result<TorrentInfo> {
    let name = get_str(fields, "name")?.to_owned();
    let state = get_str(fields, "state")?.to_owned();
    let progress = get_float(fields, "progress")?;
    let ratio = get_float(fields, "ratio")?;
    let total_seeds = get_u32(fields, "total_seeds")?;
    let num_seeds = get_u32(fields, "num_seeds")?;
    let time_added = get_int(fields, "time_added")?;
    let total_done = get_u64(fields, "total_done")?;
    let total_uploaded = get_u64(fields, "total_uploaded")?;
    let is_finished = get_bool(fields, "is_finished")?;
    let download_location = get_str(fields, "download_location")?.to_owned();

    Ok(TorrentInfo {
        info_hash: String::from(info_hash),
        name,
        state,
        progress,
        ratio,
        total_seeds,
        num_seeds,
        time_added,
        total_done,
        total_uploaded,
        is_finished,
        download_location,
    })
}

fn get_str<'a>(
    fields: &'a BTreeMap<RencodeValue, RencodeValue>,
    key: &str,
) -> anyhow::Result<&'a str> {
    match fields.get(&RencodeValue::Str(String::from(key))) {
        Some(RencodeValue::Str(s)) => Ok(s.as_str()),
        Some(other) => Err(anyhow!("field `{key}` is not a string: {other:?}")),
        None => Err(anyhow!("missing field `{key}`")),
    }
}

fn get_float(fields: &BTreeMap<RencodeValue, RencodeValue>, key: &str) -> anyhow::Result<f64> {
    match fields.get(&RencodeValue::Str(String::from(key))) {
        Some(RencodeValue::Float(f)) => Ok(*f),
        Some(RencodeValue::Int(i)) => Ok(*i as f64),
        Some(other) => Err(anyhow!("field `{key}` is not a number: {other:?}")),
        None => Err(anyhow!("missing field `{key}`")),
    }
}

fn get_int(fields: &BTreeMap<RencodeValue, RencodeValue>, key: &str) -> anyhow::Result<i64> {
    match fields.get(&RencodeValue::Str(String::from(key))) {
        Some(RencodeValue::Int(i)) => Ok(*i),
        Some(RencodeValue::Float(f)) => Ok(*f as i64),
        Some(other) => Err(anyhow!("field `{key}` is not an int: {other:?}")),
        None => Err(anyhow!("missing field `{key}`")),
    }
}

fn get_bool(fields: &BTreeMap<RencodeValue, RencodeValue>, key: &str) -> anyhow::Result<bool> {
    match fields.get(&RencodeValue::Str(String::from(key))) {
        Some(RencodeValue::Bool(b)) => Ok(*b),
        Some(other) => Err(anyhow!("field `{key}` is not a bool: {other:?}")),
        None => Err(anyhow!("missing field `{key}`")),
    }
}

fn get_u32(fields: &BTreeMap<RencodeValue, RencodeValue>, key: &str) -> anyhow::Result<u32> {
    let raw = get_int(fields, key)?;
    u32::try_from(raw).map_err(|_| anyhow!("field `{key}` out of u32 range: {raw}"))
}

fn get_u64(fields: &BTreeMap<RencodeValue, RencodeValue>, key: &str) -> anyhow::Result<u64> {
    let raw = get_int(fields, key)?;
    u64::try_from(raw).map_err(|_| anyhow!("field `{key}` is negative: {raw}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_full_fields() -> BTreeMap<RencodeValue, RencodeValue> {
        let mut fields = BTreeMap::new();
        fields.insert(
            RencodeValue::Str(String::from("name")),
            RencodeValue::Str(String::from("torrent-one")),
        );
        fields.insert(
            RencodeValue::Str(String::from("state")),
            RencodeValue::Str(String::from("Seeding")),
        );
        fields.insert(
            RencodeValue::Str(String::from("progress")),
            RencodeValue::Float(100.0),
        );
        fields.insert(
            RencodeValue::Str(String::from("ratio")),
            RencodeValue::Float(2.5),
        );
        fields.insert(
            RencodeValue::Str(String::from("total_seeds")),
            RencodeValue::Int(10),
        );
        fields.insert(
            RencodeValue::Str(String::from("num_seeds")),
            RencodeValue::Int(5),
        );
        fields.insert(
            RencodeValue::Str(String::from("time_added")),
            RencodeValue::Int(1_700_000_000),
        );
        fields.insert(
            RencodeValue::Str(String::from("total_done")),
            RencodeValue::Int(1_048_576),
        );
        fields.insert(
            RencodeValue::Str(String::from("total_uploaded")),
            RencodeValue::Int(2_097_152),
        );
        fields.insert(
            RencodeValue::Str(String::from("is_finished")),
            RencodeValue::Bool(true),
        );
        fields.insert(
            RencodeValue::Str(String::from("download_location")),
            RencodeValue::Str(String::from("/data")),
        );
        fields
    }

    #[test]
    fn when_get_torrents_response_then_dict_is_parsed_into_vec() {
        let fields = make_full_fields();

        let info =
            parse_torrent("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111", &fields).expect("parse");
        assert_eq!(info.name, "torrent-one");
        assert_eq!(info.state, "Seeding");
        assert!((info.progress - 100.0).abs() < f64::EPSILON);
        assert!((info.ratio - 2.5).abs() < f64::EPSILON);
        assert_eq!(info.total_seeds, 10);
        assert_eq!(info.num_seeds, 5);
        assert_eq!(info.time_added, 1_700_000_000);
        assert_eq!(info.total_done, 1_048_576);
        assert_eq!(info.total_uploaded, 2_097_152);
        assert!(info.is_finished);
        assert_eq!(info.download_location, "/data");
    }

    #[test]
    fn when_torrents_unsorted_then_parse_sorts_by_info_hash() {
        let fields = make_full_fields();

        let mut result_dict = BTreeMap::new();
        result_dict.insert(
            RencodeValue::Str(String::from("zzzz")),
            RencodeValue::Dict(fields.clone()),
        );
        result_dict.insert(
            RencodeValue::Str(String::from("aaaa")),
            RencodeValue::Dict(fields),
        );

        let mut entries: Vec<String> = result_dict
            .iter()
            .filter_map(|(k, _)| match k {
                RencodeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        entries.sort();

        assert_eq!(entries, vec!["aaaa", "zzzz"]);
    }

    #[test]
    fn when_total_seeds_out_of_u32_range_then_parse_returns_error() {
        let mut fields = make_full_fields();
        fields.insert(
            RencodeValue::Str(String::from("total_seeds")),
            RencodeValue::Int(i64::from(u32::MAX) + 1),
        );

        let result = parse_torrent("deadbeef", &fields);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("total_seeds"), "got: {err}");
    }
}
