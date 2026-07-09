use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::Deserialize;
use std::fmt;

/// Private helper: deserializes `Vec<u8>` via `deserialize_bytes` instead of `deserialize_seq`.
/// Needed because `RencodeValue::Bytes` only implements `deserialize_bytes`, not `deserialize_seq`.
struct ByteBuf(Vec<u8>);

impl<'de> Deserialize<'de> for ByteBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ByteBufVisitor;

        impl<'de> Visitor<'de> for ByteBufVisitor {
            type Value = ByteBuf;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("byte buffer")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ByteBuf(v.to_vec()))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ByteBuf(v))
            }
        }

        deserializer.deserialize_bytes(ByteBufVisitor)
    }
}

/// Return of `core.add_torrent_file` / `core.add_torrent_file_async` / `core.add_torrent_url`.
/// `None` on failure, `Some(torrent_id)` on success.
pub type AddTorrentFileResult = Option<String>;

/// Return of `core.add_torrent_files`.
/// Each element is an error message for a torrent that failed to add.
/// Empty list = all succeeded.
pub type AddTorrentFilesResult = Vec<String>;

/// Return of `core.remove_torrents`.
/// Each tuple is `(torrent_id, error_message)` for failed removals.
/// Empty list = all succeeded.
pub type RemoveTorrentsResult = Vec<(String, String)>;

/// Return of `core.get_magnet_uri`.
pub type GetMagnetUriResult = String;

/// Return of `core.prefetch_magnet_metadata`.
/// Tuple of `(torrent_id, metadata)` where `metadata` is bencoded torrent data.
#[derive(Debug, Clone, PartialEq)]
pub struct PrefetchMagnetResult {
    /// Torrent id (hex string).
    pub torrent_id: String,
    /// Bencoded torrent data.
    pub metadata: Vec<u8>,
}

impl<'de> Deserialize<'de> for PrefetchMagnetResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PrefetchVisitor;

        impl<'de> Visitor<'de> for PrefetchVisitor {
            type Value = PrefetchMagnetResult;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 2-element tuple (torrent_id, metadata)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let torrent_id: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let metadata: ByteBuf = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(PrefetchMagnetResult {
                    torrent_id,
                    metadata: metadata.0,
                })
            }
        }

        deserializer.deserialize_seq(PrefetchVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rpc_rencode::RencodeValue;

    #[test]
    fn when_prefetch_magnet_result_from_tuple_then_fields_populate() {
        let value = RencodeValue::List(vec![
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
            RencodeValue::Bytes(b"bencoded-data".to_vec()),
        ]);

        let result: PrefetchMagnetResult =
            PrefetchMagnetResult::deserialize(&value).expect("deserialize");

        assert_eq!(
            result.torrent_id,
            "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111"
        );
        assert_eq!(result.metadata, b"bencoded-data");
    }

    #[test]
    fn when_remove_torrents_result_from_list_then_deserialized() {
        let value = RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
            RencodeValue::Str("error message".into()),
        ])]);

        let result: RemoveTorrentsResult =
            RemoveTorrentsResult::deserialize(&value).expect("deserialize");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
        assert_eq!(result[0].1, "error message");
    }

    #[test]
    fn when_add_torrent_file_result_some_then_deserialized() {
        let value = RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into());

        let result: AddTorrentFileResult =
            AddTorrentFileResult::deserialize(&value).expect("deserialize");

        assert_eq!(
            result,
            Some("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into())
        );
    }

    #[test]
    fn when_add_torrent_file_result_none_then_deserialized() {
        let value = RencodeValue::None;

        let result: AddTorrentFileResult =
            AddTorrentFileResult::deserialize(&value).expect("deserialize");

        assert_eq!(result, None);
    }
}
