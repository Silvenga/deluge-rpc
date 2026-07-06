use serde::Deserialize;

/// Input and return shape for `core.get_completion_paths()`.
///
/// The input dict contains `completion_text` and `show_hidden_files`. The
/// return dict passes those through unchanged and adds a `paths` field with
/// sorted matching directory paths (each with a trailing `/`).
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CompletionPaths {
    pub completion_text: String,
    pub show_hidden_files: bool,
    #[serde(default)]
    pub paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rencode::RencodeValue;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    fn make_completion_paths_dict() -> RencodeValue {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("completion_text".into()),
            RencodeValue::Str("/home/".into()),
        );
        map.insert(
            RencodeValue::Str("show_hidden_files".into()),
            RencodeValue::Bool(false),
        );
        map.insert(
            RencodeValue::Str("paths".into()),
            RencodeValue::List(vec![
                RencodeValue::Str("/home/user/".into()),
                RencodeValue::Str("/home/other/".into()),
            ]),
        );
        RencodeValue::Dict(map)
    }

    #[test]
    fn when_completion_paths_then_paths_returned() {
        let value = make_completion_paths_dict();

        let result: CompletionPaths = CompletionPaths::deserialize(&value).expect("deserialize");

        assert_eq!(result.completion_text, "/home/");
        assert!(!result.show_hidden_files);
        assert_eq!(result.paths, vec!["/home/user/", "/home/other/"]);
    }

    #[test]
    fn when_completion_paths_no_paths_then_defaults_to_empty() {
        let mut map = BTreeMap::new();
        map.insert(
            RencodeValue::Str("completion_text".into()),
            RencodeValue::Str("/tmp/".into()),
        );
        map.insert(
            RencodeValue::Str("show_hidden_files".into()),
            RencodeValue::Bool(true),
        );
        let value = RencodeValue::Dict(map);

        let result: CompletionPaths = CompletionPaths::deserialize(&value).expect("deserialize");

        assert_eq!(result.completion_text, "/tmp/");
        assert!(result.show_hidden_files);
        assert!(result.paths.is_empty());
    }
}
