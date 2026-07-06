use crate::helpers::rencode_from_json_value;
use clap::Args;
use deluge_rpc::{DelugeClient, DelugeRpcRequest, RencodeValue};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

#[derive(Args, Debug, Clone)]
pub struct CallCommand {
    pub method: String,
    pub args_json: Option<String>,
    pub kwargs_json: Option<String>,
}

impl CallCommand {
    pub async fn run(&self, client: &DelugeClient) -> anyhow::Result<RencodeValue> {
        let parsed_args: Vec<RencodeValue> = match &self.args_json {
            Some(json_str) => {
                let json: JsonValue = serde_json::from_str(json_str)
                    .map_err(|e| anyhow::anyhow!("failed to parse args JSON: {e}"))?;
                let arr = json
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("args must be a JSON array"))?;
                arr.iter()
                    .map(rencode_from_json_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| anyhow::anyhow!("failed to convert args to RencodeValue: {e}"))?
            }
            None => vec![],
        };

        let parsed_kwargs: BTreeMap<RencodeValue, RencodeValue> = match &self.kwargs_json {
            Some(json_str) => {
                let json: JsonValue = serde_json::from_str(json_str)
                    .map_err(|e| anyhow::anyhow!("failed to parse kwargs JSON: {e}"))?;
                let obj = json
                    .as_object()
                    .ok_or_else(|| anyhow::anyhow!("kwargs must be a JSON object"))?;
                let mut map = BTreeMap::new();
                for (k, v) in obj {
                    let key = RencodeValue::Str(k.clone());
                    let val = rencode_from_json_value(v)?;
                    map.insert(key, val);
                }
                map
            }
            None => BTreeMap::new(),
        };

        let request = DelugeRpcRequest::new(&self.method)
            .with_args(parsed_args)
            .with_kwargs(parsed_kwargs);
        client
            .call(request)
            .await
            .map_err(|e| anyhow::anyhow!("RPC call to '{}' failed: {e}", self.method))
    }
}
