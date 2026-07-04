use crate::RencodeValue;
use std::collections::BTreeMap;

pub struct DelugeRpcRequest {
    pub method: String,
    pub args: Vec<RencodeValue>,
    pub kwargs: BTreeMap<RencodeValue, RencodeValue>,
}

impl DelugeRpcRequest {
    pub fn new(method: impl Into<String>) -> Self {
        DelugeRpcRequest {
            method: method.into(),
            args: vec![RencodeValue::None],
            kwargs: BTreeMap::new(),
        }
    }

    pub fn with_args(mut self, args: Vec<RencodeValue>) -> Self {
        self.args = args;
        self
    }

    pub fn with_kwargs(mut self, kwargs: BTreeMap<RencodeValue, RencodeValue>) -> Self {
        self.kwargs = kwargs;
        self
    }

    pub fn encode(self, id: u32) -> Vec<u8> {
        self.into_rencode_value(id).encode()
    }

    pub fn into_rencode_value(self, id: u32) -> RencodeValue {
        RencodeValue::List(vec![RencodeValue::List(vec![
            RencodeValue::Int(i64::from(id)),
            RencodeValue::Str(self.method),
            RencodeValue::List(self.args),
            RencodeValue::Dict(self.kwargs),
        ])])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_arg_is_empty_then_args_is_none() {
        let request = DelugeRpcRequest::new("core.get_free_space").into_rencode_value(2);
        let parts = unwrap_request(&request);
        assert_eq!(parts[0], RencodeValue::Int(2));
        match &parts[2] {
            RencodeValue::List(args) => {
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], RencodeValue::None);
            }
            _ => panic!("args is not a list"),
        }
        match &parts[3] {
            RencodeValue::Dict(map) => assert!(map.is_empty()),
            _ => panic!("kwargs is not a dict"),
        }
    }

    fn unwrap_request(value: &RencodeValue) -> Vec<RencodeValue> {
        let outer = match value {
            RencodeValue::List(items) if items.len() == 1 => &items[0],
            _ => panic!("expected 1-element outer list"),
        };
        match outer {
            RencodeValue::List(p) => p.clone(),
            _ => panic!("expected inner list"),
        }
    }
}
