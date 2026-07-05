use crate::client::caller::RpcCaller;
use crate::models::AccountInfo;
use crate::protocol::extract_single;
use crate::protocol::DelugeRpcRequest;
use crate::rencode::RencodeValue;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::BTreeMap;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait CoreAccountRpc: Send + Sync {
    async fn get_known_accounts(&self) -> anyhow::Result<Vec<AccountInfo>>;
    async fn create_account(
        &self,
        username: &str,
        password: &str,
        auth_level: &str,
    ) -> anyhow::Result<bool>;
    async fn update_account(
        &self,
        username: &str,
        password: &str,
        auth_level: &str,
    ) -> anyhow::Result<bool>;
    async fn remove_account(&self, username: &str) -> anyhow::Result<bool>;
    async fn get_auth_levels_mappings(
        &self,
    ) -> anyhow::Result<(BTreeMap<String, i64>, BTreeMap<i64, String>)>;
}

pub struct CoreAccountClient {
    caller: RpcCaller,
}

impl CoreAccountClient {
    pub(crate) fn new(caller: RpcCaller) -> Self {
        Self { caller }
    }
}

impl Clone for CoreAccountClient {
    fn clone(&self) -> Self {
        Self {
            caller: self.caller.clone(),
        }
    }
}

#[async_trait]
impl CoreAccountRpc for CoreAccountClient {
    async fn get_known_accounts(&self) -> anyhow::Result<Vec<AccountInfo>> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_known_accounts"))
            .await
            .context("core.get_known_accounts RPC failed")?;
        let value = extract_single(&result)?;
        Vec::<AccountInfo>::deserialize(&value).context("deserializing accounts")
    }

    async fn create_account(
        &self,
        username: &str,
        password: &str,
        auth_level: &str,
    ) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.create_account").with_args(vec![
                RencodeValue::Str(username.to_owned()),
                RencodeValue::Str(password.to_owned()),
                RencodeValue::Str(auth_level.to_owned()),
            ]))
            .await
            .context("core.create_account RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "core.create_account returned non-bool value: {other:?}"
            )),
        }
    }

    async fn update_account(
        &self,
        username: &str,
        password: &str,
        authlevel: &str,
    ) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.update_account").with_args(vec![
                RencodeValue::Str(username.to_owned()),
                RencodeValue::Str(password.to_owned()),
                RencodeValue::Str(authlevel.to_owned()),
            ]))
            .await
            .context("core.update_account RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "core.update_account returned non-bool value: {other:?}"
            )),
        }
    }

    async fn remove_account(&self, username: &str) -> anyhow::Result<bool> {
        let result = self
            .caller
            .rpc_call(
                DelugeRpcRequest::new("core.remove_account")
                    .with_args(vec![RencodeValue::Str(username.to_owned())]),
            )
            .await
            .context("core.remove_account RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::Bool(b) => Ok(b),
            other => Err(anyhow!(
                "core.remove_account returned non-bool value: {other:?}"
            )),
        }
    }

    async fn get_auth_levels_mappings(
        &self,
    ) -> anyhow::Result<(BTreeMap<String, i64>, BTreeMap<i64, String>)> {
        let result = self
            .caller
            .rpc_call(DelugeRpcRequest::new("core.get_auth_levels_mappings"))
            .await
            .context("core.get_auth_levels_mappings RPC failed")?;
        let value = extract_single(&result)?;
        match value {
            RencodeValue::List(items) if items.len() == 2 => {
                let name_to_int: BTreeMap<String, i64> =
                    BTreeMap::<String, i64>::deserialize(&items[0])
                        .context("deserializing name->int mapping")?;
                let int_to_name: BTreeMap<i64, String> =
                    BTreeMap::<i64, String>::deserialize(&items[1])
                        .context("deserializing int->name mapping")?;
                Ok((name_to_int, int_to_name))
            }
            other => Err(anyhow!(
                "core.get_auth_levels_mappings returned unexpected shape: {other:?}"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rencode::RencodeValue;
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
