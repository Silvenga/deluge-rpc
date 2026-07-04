use serde::Deserialize;

/// User account info returned by `core.get_known_accounts()`.
///
/// Each account has a username, password hash, auth level name, and auth level
/// integer.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AccountInfo {
    pub username: String,
    pub password: String,
    pub authlevel: String,
    pub authlevel_int: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    fn make_account_dict() -> RencodeValue {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("username".into()),
            RencodeValue::Str("admin".into()),
        );
        map.insert(
            RencodeValue::Str("password".into()),
            RencodeValue::Str("hashed_password".into()),
        );
        map.insert(
            RencodeValue::Str("authlevel".into()),
            RencodeValue::Str("ADMIN".into()),
        );
        map.insert(
            RencodeValue::Str("authlevel_int".into()),
            RencodeValue::Int(10),
        );
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_account_dict_then_authlevel_parsed() {
        let value = make_account_dict();

        let result: AccountInfo = AccountInfo::deserialize(&value).expect("deserialize");

        assert_eq!(result.username, "admin");
        assert_eq!(result.password, "hashed_password");
        assert_eq!(result.authlevel, "ADMIN");
        assert_eq!(result.authlevel_int, 10);
    }

    #[test]
    fn when_account_dict_readonly_then_authlevel_parsed() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("username".into()),
            RencodeValue::Str("viewer".into()),
        );
        map.insert(
            RencodeValue::Str("password".into()),
            RencodeValue::Str("hash".into()),
        );
        map.insert(
            RencodeValue::Str("authlevel".into()),
            RencodeValue::Str("READONLY".into()),
        );
        map.insert(
            RencodeValue::Str("authlevel_int".into()),
            RencodeValue::Int(1),
        );
        let value = RencodeValue::Dict(map);

        let result: AccountInfo = AccountInfo::deserialize(&value).expect("deserialize");

        assert_eq!(result.username, "viewer");
        assert_eq!(result.authlevel, "READONLY");
        assert_eq!(result.authlevel_int, 1);
    }
}
