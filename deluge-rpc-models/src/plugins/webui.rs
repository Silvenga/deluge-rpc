use serde::{Deserialize, Serialize};

/// Configuration for the WebUi plugin.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct WebUiConfig {
    /// Whether the web UI server is enabled.
    pub enabled: bool,
    /// Whether the web UI uses SSL.
    pub ssl: bool,
    /// Port for the web UI server.
    pub port: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rpc_rencode::RencodeValue;
    use std::collections::BTreeMap;

    fn make_dict(entries: Vec<(&str, RencodeValue)>) -> RencodeValue {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(RencodeValue::Str(k.into()), v);
        }
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_webui_config_dict_then_fields_populate() {
        let value = make_dict(vec![
            ("enabled", RencodeValue::Bool(true)),
            ("ssl", RencodeValue::Bool(false)),
            ("port", RencodeValue::Int(8112)),
        ]);

        let result: WebUiConfig = WebUiConfig::deserialize(&value).expect("deserialize");

        assert_eq!(
            result,
            WebUiConfig {
                enabled: true,
                ssl: false,
                port: 8112,
            }
        );
    }
}
