use serde::Deserialize;

/// Result of `core.create_torrent()`.
///
/// Returns a tuple of `(filename, filedump)` where `filedump` is base64-encoded
/// bencoded torrent data.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CreateTorrentResult {
    pub filename: String,
    pub filedump: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_create_torrent_list_then_result_parsed() {
        let value = RencodeValue::List(vec![
            RencodeValue::Str("my_torrent.torrent".into()),
            RencodeValue::Str("ZDg6YW5ub3VuY2U=".into()),
        ]);

        let result: CreateTorrentResult =
            CreateTorrentResult::deserialize(&value).expect("deserialize");

        assert_eq!(result.filename, "my_torrent.torrent");
        assert_eq!(result.filedump, "ZDg6YW5ub3VuY2U=");
    }
}
