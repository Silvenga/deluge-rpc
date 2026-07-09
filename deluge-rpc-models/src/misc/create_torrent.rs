use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, de};
use std::fmt;

/// Result of `core.create_torrent()`.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTorrentResult {
    /// Torrent filename.
    pub filename: String,
    /// Base64-encoded bencoded torrent data.
    pub file_dump: String,
}

impl<'de> Deserialize<'de> for CreateTorrentResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CreateVisitor;

        impl<'de> Visitor<'de> for CreateVisitor {
            type Value = CreateTorrentResult;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 2-element tuple (torrent_id, filedump)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let filename: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let file_dump: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(CreateTorrentResult {
                    filename,
                    file_dump,
                })
            }
        }

        deserializer.deserialize_seq(CreateVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rpc_rencode::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_create_torrent_result_from_tuple_then_fields_populate() {
        let value = RencodeValue::List(vec![
            RencodeValue::Str("aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111".into()),
            RencodeValue::Str("base64-encoded-data".into()),
        ]);

        let result: CreateTorrentResult =
            CreateTorrentResult::deserialize(&value).expect("deserialize");

        assert_eq!(result.filename, "aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111");
        assert_eq!(result.file_dump, "base64-encoded-data");
    }
}
