use crate::DelugeRpcError;
use crate::RencodeValue;
use crate::client::dispatcher::DelugeClientDispatcher;
use crate::models::AccountInfo;
use crate::protocol::DelugeRpcRequest;
use crate::protocol::extract_single;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::BTreeMap;

/// RPC methods for core.* account management.
#[async_trait]
pub trait CoreAccountRpc: Send + Sync {
    /// Returns all known user accounts.
    async fn get_known_accounts(&self) -> Result<Vec<AccountInfo>, DelugeRpcError>;
    /// Creates a new user account.
    async fn create_account(
        &self,
        username: &str,
        password: &str,
        auth_level: &str,
    ) -> Result<bool, DelugeRpcError>;
    /// Updates an existing account's password and/or auth level.
    async fn update_account(
        &self,
        username: &str,
        password: &str,
        auth_level: &str,
    ) -> Result<bool, DelugeRpcError>;
    /// Removes a user account.
    async fn remove_account(&self, username: &str) -> Result<bool, DelugeRpcError>;
    /// Returns auth level name-to-int and int-to-name mappings.
    async fn get_auth_levels_mappings(
        &self,
    ) -> Result<(BTreeMap<String, i64>, BTreeMap<i64, String>), DelugeRpcError>;
}

/// Client for core.* account RPC methods.
pub struct CoreAccountClient {
    dispatcher: DelugeClientDispatcher,
}

impl CoreAccountClient {
    pub(crate) fn new(dispatcher: DelugeClientDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Clone for CoreAccountClient {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

#[async_trait]
impl CoreAccountRpc for CoreAccountClient {
    async fn get_known_accounts(&self) -> Result<Vec<AccountInfo>, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.get_known_accounts"))
            .await?;
        let value = extract_single(&result)?;
        Ok(Vec::<AccountInfo>::deserialize(&value)?)
    }

    async fn create_account(
        &self,
        username: &str,
        password: &str,
        auth_level: &str,
    ) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.create_account").with_args(vec![
                RencodeValue::Str(username.to_owned()),
                RencodeValue::Str(password.to_owned()),
                RencodeValue::Str(auth_level.to_owned()),
            ]))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.create_account".into(),
                value: other,
            }),
        }
    }

    async fn update_account(
        &self,
        username: &str,
        password: &str,
        authlevel: &str,
    ) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.update_account").with_args(vec![
                RencodeValue::Str(username.to_owned()),
                RencodeValue::Str(password.to_owned()),
                RencodeValue::Str(authlevel.to_owned()),
            ]))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.update_account".into(),
                value: other,
            }),
        }
    }

    async fn remove_account(&self, username: &str) -> Result<bool, DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(
                DelugeRpcRequest::new("core.remove_account")
                    .with_args(vec![RencodeValue::Str(username.to_owned())]),
            )
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.remove_account".into(),
                value: other,
            }),
        }
    }

    async fn get_auth_levels_mappings(
        &self,
    ) -> Result<(BTreeMap<String, i64>, BTreeMap<i64, String>), DelugeRpcError> {
        let result = self
            .dispatcher
            .dispatch(DelugeRpcRequest::new("core.get_auth_levels_mappings"))
            .await?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::List(items) if items.len() == 2 => {
                let name_to_int: BTreeMap<String, i64> =
                    BTreeMap::<String, i64>::deserialize(&items[0])?;
                let int_to_name: BTreeMap<i64, String> =
                    BTreeMap::<i64, String>::deserialize(&items[1])?;
                Ok((name_to_int, int_to_name))
            }
            other => Err(DelugeRpcError::UnexpectedResponseType {
                method: "core.get_auth_levels_mappings returned unexpected shape".into(),
                value: other,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use std::collections::BTreeMap;

    #[test]
    fn when_core_get_known_accounts_then_vec_account_info() {
        let mut account = BTreeMap::new();
        account.insert(
            RencodeValue::Str("username".into()),
            RencodeValue::Str("admin".into()),
        );
        account.insert(
            RencodeValue::Str("password".into()),
            RencodeValue::Str("hash".into()),
        );
        account.insert(
            RencodeValue::Str("authlevel".into()),
            RencodeValue::Str("ADMIN".into()),
        );
        account.insert(
            RencodeValue::Str("authlevel_int".into()),
            RencodeValue::Int(10),
        );
        let response = RencodeValue::List(vec![RencodeValue::Dict(account)]);

        let value = extract_single(&response).expect("extract");
        let accounts: Vec<AccountInfo> =
            Vec::<AccountInfo>::deserialize(&value).expect("deserialize");
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].username, "admin");
        assert_eq!(accounts[0].auth_level, "ADMIN");
        assert_eq!(accounts[0].auth_level_int, 10);
    }

    #[test]
    fn when_core_create_account_then_bool() {
        let response = RencodeValue::Bool(true);
        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::Bool(b) => assert!(b),
            other => panic!("expected bool, got {other:?}"),
        }
    }

    #[test]
    fn when_core_get_auth_levels_mappings_then_tuple() {
        let mut name_to_int = BTreeMap::new();
        name_to_int.insert(RencodeValue::Str("NONE".into()), RencodeValue::Int(0));
        name_to_int.insert(RencodeValue::Str("ADMIN".into()), RencodeValue::Int(10));
        let mut int_to_name = BTreeMap::new();
        int_to_name.insert(RencodeValue::Int(0), RencodeValue::Str("NONE".into()));
        int_to_name.insert(RencodeValue::Int(10), RencodeValue::Str("ADMIN".into()));
        let response = RencodeValue::List(vec![
            RencodeValue::Dict(name_to_int),
            RencodeValue::Dict(int_to_name),
        ]);

        let value = extract_single(&response).expect("extract");
        match value {
            RencodeValue::List(items) => {
                assert_eq!(items.len(), 2);
            }
            other => panic!("expected list, got {other:?}"),
        }
    }
}
