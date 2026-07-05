use clap::Args;
use deluge_rpc::{RencodeValue, RpcCaller};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

use crate::record::response_to_tagged_json;

#[derive(Args, Debug, Clone)]
pub struct CallArgs {
    pub method: String,

    pub args_json: Option<String>,

    pub kwargs_json: Option<String>,
}

pub async fn run_call(
    client_rpc: &RpcCaller,
    args: &CallArgs,
) -> anyhow::Result<(
    String,
    Vec<RencodeValue>,
    BTreeMap<RencodeValue, RencodeValue>,
    RencodeValue,
)> {
    let parsed_args: Vec<RencodeValue> = match &args.args_json {
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

    let parsed_kwargs: BTreeMap<RencodeValue, RencodeValue> = match &args.kwargs_json {
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

    let request = deluge_rpc::DelugeRpcRequest::new(&args.method)
        .with_args(parsed_args.clone())
        .with_kwargs(parsed_kwargs.clone());
    let response = client_rpc
        .rpc_call(request)
        .await
        .map_err(|e| anyhow::anyhow!("RPC call to '{}' failed: {e}", args.method))?;

    Ok((
        args.method.clone(),
        parsed_args,
        parsed_kwargs,
        response.clone(),
    ))
}

#[expect(clippy::print_stdout, reason = "CLI prints call result to stdout")]
pub fn print_call_result(response: &RencodeValue) {
    let tagged = response_to_tagged_json(response);
    let output = serde_json::to_string_pretty(&tagged).unwrap_or_else(|_| "{}".to_owned());
    println!("{output}");
}

pub fn rencode_from_json_value(json: &JsonValue) -> anyhow::Result<RencodeValue> {
    deluge_rpc::from_json(json)
        .map_err(|e| anyhow::anyhow!("failed to convert JSON to RencodeValue: {e}"))
}
