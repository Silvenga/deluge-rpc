use crate::RencodeValue;
use crate::rencode::RencodeError;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TorrentInfo {
    #[serde(skip)]
    pub info_hash: String,
    pub name: String,
    pub state: String,
    pub progress: f64,
    pub ratio: f64,
    pub total_seeds: u32,
    pub num_seeds: u32,
    pub time_added: i64,
    pub total_done: u64,
    pub total_uploaded: u64,
    pub is_finished: bool,
    pub download_location: String,
}

impl TorrentInfo {
    pub fn from(info_hash: impl Into<String>, fields: &RencodeValue) -> Result<Self, RencodeError> {
        Ok(Self {
            info_hash: info_hash.into(),
            name: fields.get_str("name")?.to_owned(),
            state: fields.get_str("state")?.to_owned(),
            progress: fields.get_num("progress")?,
            ratio: fields.get_num("ratio")?,
            total_seeds: get_u32(fields, "total_seeds")?,
            num_seeds: get_u32(fields, "num_seeds")?,
            time_added: fields.get_int("time_added")?,
            total_done: get_u64(fields, "total_done")?,
            total_uploaded: get_u64(fields, "total_uploaded")?,
            is_finished: fields.get_bool("is_finished")?,
            download_location: fields.get_str("download_location")?.to_owned(),
        })
    }
}

fn get_u32(fields: &RencodeValue, key: &str) -> Result<u32, RencodeError> {
    fields
        .get_int(key)?
        .try_into()
        .map_err(|_| RencodeError::NumberParse(format!("field `{key}` out of u32 range")))
}

fn get_u64(fields: &RencodeValue, key: &str) -> Result<u64, RencodeError> {
    fields
        .get_int(key)?
        .try_into()
        .map_err(|_| RencodeError::NumberParse(format!("field `{key}` is negative")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_full_fields() -> RencodeValue {
        let mut fields = std::collections::BTreeMap::new();
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
        RencodeValue::Dict(fields)
    }

    #[test]
    fn when_get_torrents_response_then_dict_is_parsed_into_vec() {
        let fields = make_full_fields();

        let info =
            TorrentInfo::from("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111", &fields).expect("parse");
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
    fn when_total_seeds_out_of_u32_range_then_parse_returns_error() {
        let mut fields = match make_full_fields() {
            RencodeValue::Dict(m) => m,
            _ => unreachable!(),
        };
        fields.insert(
            RencodeValue::Str(String::from("total_seeds")),
            RencodeValue::Int(i64::from(u32::MAX) + 1),
        );

        let result = TorrentInfo::from("deadbeef", &RencodeValue::Dict(fields));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("total_seeds"), "got: {err}");
    }
}
