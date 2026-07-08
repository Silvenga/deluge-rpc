use crate::Interaction;
use deluge_rpc_client::RencodeValue;
use std::sync::Mutex;

/// Matches incoming RPC requests against recorded interactions.
///
/// Tries an exact (method + args) match first, then falls back to
/// a method-only match. Each interaction is consumed at most once.
pub struct Matcher {
    entries: Mutex<Vec<(Interaction, bool)>>,
}

impl Matcher {
    /// Create a new matcher from a list of recorded interactions.
    pub fn new(interactions: Vec<Interaction>) -> Self {
        let entries = interactions.into_iter().map(|i| (i, false)).collect();
        Self {
            entries: Mutex::new(entries),
        }
    }

    /// Find and consume an interaction matching the given method and args.
    ///
    /// Prefers an exact (method + args) match. Falls back to any unconsumed
    /// interaction with the same method name.
    pub fn find_match(&self, method: &str, args: &RencodeValue) -> Option<Interaction> {
        let mut entries = self.entries.lock().expect("matcher mutex poisoned");

        if let Some(idx) = entries.iter().position(|(e, consumed)| {
            !consumed && e.request.method == method && &e.request.args == args
        }) {
            entries[idx].1 = true;
            return Some(entries[idx].0.clone());
        }

        if let Some(idx) = entries
            .iter()
            .position(|(e, consumed)| !consumed && e.request.method == method)
        {
            entries[idx].1 = true;
            return Some(entries[idx].0.clone());
        }

        None
    }

    /// The list of RPC methods that have been matched and consumed.
    pub fn consumed_methods(&self) -> Vec<String> {
        let entries = self.entries.lock().expect("matcher mutex poisoned");
        entries
            .iter()
            .filter(|(_, consumed)| *consumed)
            .map(|(e, _)| e.request.method.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InteractionRequest, InteractionResponse};
    use deluge_rpc_client::RencodeValue;

    fn make_interaction(method: &str, args: RencodeValue, response_value: &str) -> Interaction {
        Interaction {
            request: InteractionRequest {
                method: method.into(),
                args,
                kwargs: RencodeValue::List(vec![]),
            },
            response: InteractionResponse::Ok {
                value: RencodeValue::Str(response_value.into()),
            },
        }
    }

    #[test]
    fn when_exact_method_and_args_match_then_returns_interaction() {
        let interactions = vec![make_interaction(
            "test.method",
            RencodeValue::List(vec![RencodeValue::Int(1)]),
            "first",
        )];
        let matcher = Matcher::new(interactions);

        let result = matcher.find_match(
            "test.method",
            &RencodeValue::List(vec![RencodeValue::Int(1)]),
        );
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().response,
            InteractionResponse::Ok {
                value: RencodeValue::Str("first".into())
            }
        );
    }

    #[test]
    fn when_entry_consumed_then_not_rematched() {
        let interactions = vec![make_interaction(
            "test.method",
            RencodeValue::List(vec![RencodeValue::Int(1)]),
            "only",
        )];
        let matcher = Matcher::new(interactions);

        let first = matcher.find_match(
            "test.method",
            &RencodeValue::List(vec![RencodeValue::Int(1)]),
        );
        assert!(first.is_some());

        let second = matcher.find_match(
            "test.method",
            &RencodeValue::List(vec![RencodeValue::Int(1)]),
        );
        assert!(second.is_none());
    }

    #[test]
    fn when_no_exact_match_then_fallback_to_unmatched_for_method() {
        let interactions = vec![
            make_interaction(
                "test.method",
                RencodeValue::List(vec![RencodeValue::Int(1)]),
                "first",
            ),
            make_interaction(
                "test.method",
                RencodeValue::List(vec![RencodeValue::Int(2)]),
                "second",
            ),
        ];
        let matcher = Matcher::new(interactions);

        let result = matcher.find_match(
            "test.method",
            &RencodeValue::List(vec![RencodeValue::Int(99)]),
        );
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().response,
            InteractionResponse::Ok {
                value: RencodeValue::Str("first".into())
            }
        );
    }

    #[test]
    fn when_fallback_consumes_then_next_match_goes_to_second() {
        let interactions = vec![
            make_interaction(
                "test.method",
                RencodeValue::List(vec![RencodeValue::Int(1)]),
                "first",
            ),
            make_interaction(
                "test.method",
                RencodeValue::List(vec![RencodeValue::Int(2)]),
                "second",
            ),
        ];
        let matcher = Matcher::new(interactions);

        let r1 = matcher.find_match(
            "test.method",
            &RencodeValue::List(vec![RencodeValue::Int(99)]),
        );
        assert_eq!(
            r1.unwrap().response,
            InteractionResponse::Ok {
                value: RencodeValue::Str("first".into())
            }
        );

        let r2 = matcher.find_match(
            "test.method",
            &RencodeValue::List(vec![RencodeValue::Int(99)]),
        );
        assert_eq!(
            r2.unwrap().response,
            InteractionResponse::Ok {
                value: RencodeValue::Str("second".into())
            }
        );
    }

    #[test]
    fn when_unknown_method_then_none() {
        let interactions = vec![make_interaction(
            "test.method",
            RencodeValue::List(vec![RencodeValue::Int(1)]),
            "only",
        )];
        let matcher = Matcher::new(interactions);

        let result = matcher.find_match("unknown.method", &RencodeValue::List(vec![]));
        assert!(result.is_none());
    }

    #[test]
    fn when_empty_interactions_then_none() {
        let matcher = Matcher::new(vec![]);
        let result = matcher.find_match("any.method", &RencodeValue::List(vec![]));
        assert!(result.is_none());
    }
}
