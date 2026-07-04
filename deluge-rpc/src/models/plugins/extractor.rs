use serde::{Deserialize, Serialize};

/// Configuration for the Extractor plugin.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExtractorConfig {
    /// Destination path for extracted files. Empty means extract in-place.
    pub extract_path: String,
    /// Whether to extract into a folder named after the torrent.
    pub use_name_folder: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use std::collections::BTreeMap;

    fn make_dict(entries: Vec<(&str, RencodeValue)>) -> RencodeValue {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(RencodeValue::Str(k.into()), v);
        }
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_extractor_config_dict_then_fields_populate() {
        let value = make_dict(vec![
            ("extract_path", RencodeValue::Str("/tmp/extract".into())),
            ("use_name_folder", RencodeValue::Bool(false)),
        ]);

        let result: ExtractorConfig = ExtractorConfig::deserialize(&value).expect("deserialize");

        assert_eq!(
            result,
            ExtractorConfig {
                extract_path: "/tmp/extract".into(),
                use_name_folder: false,
            }
        );
    }
}
