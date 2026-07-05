use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct ProxyConfig {
    /// Proxy type: 0=None, 1=Socks4, 2=Socks5, 3=Socks5+Auth, 4=HTTP, 5=HTTP+Auth, 6=I2P
    #[serde(rename = "type")]
    pub proxy_type: i64,
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub port: i64,
    pub proxy_hostnames: bool,
    pub proxy_peer_connections: bool,
    pub proxy_tracker_connections: bool,
    pub force_proxy: bool,
    pub anonymous_mode: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    fn make_proxy_dict() -> RencodeValue {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str("type".into()), RencodeValue::Int(2));
        map.insert(
            RencodeValue::Str("hostname".into()),
            RencodeValue::Str("proxy.example.com".into()),
        );
        map.insert(
            RencodeValue::Str("username".into()),
            RencodeValue::Str("user".into()),
        );
        map.insert(
            RencodeValue::Str("password".into()),
            RencodeValue::Str("pass".into()),
        );
        map.insert(RencodeValue::Str("port".into()), RencodeValue::Int(1080));
        map.insert(
            RencodeValue::Str("proxy_hostnames".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("proxy_peer_connections".into()),
            RencodeValue::Bool(true),
        );
        map.insert(
            RencodeValue::Str("proxy_tracker_connections".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("force_proxy".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("anonymous_mode".into()),
            RencodeValue::Bool(false),
        );
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_proxy_nested_then_proxy_config_parsed() {
        let value = make_proxy_dict();

        let result: ProxyConfig = ProxyConfig::deserialize(&value).expect("deserialize");

        assert_eq!(result.proxy_type, 2);
        assert_eq!(result.hostname, "proxy.example.com");
        assert_eq!(result.username, "user");
        assert_eq!(result.password, "pass");
        assert_eq!(result.port, 1080);
        assert!(result.proxy_hostnames);
        assert!(result.proxy_peer_connections);
        assert!(!result.proxy_tracker_connections);
        assert!(!result.force_proxy);
        assert!(!result.anonymous_mode);
    }
}
