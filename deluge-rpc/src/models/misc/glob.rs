/// Result of `core.glob(path)`.
///
/// Returns a list of filesystem paths matching the glob pattern.
pub type GlobResult = Vec<String>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use serde::Deserialize;

    #[test]
    fn when_glob_list_then_result_parsed() {
        let value = RencodeValue::List(vec![
            RencodeValue::Str("/downloads/file1.mkv".into()),
            RencodeValue::Str("/downloads/file2.mkv".into()),
        ]);

        let result: GlobResult = GlobResult::deserialize(&value).expect("deserialize");

        assert_eq!(result, vec!["/downloads/file1.mkv", "/downloads/file2.mkv"]);
    }

    #[test]
    fn when_glob_empty_list_then_result_empty() {
        let value = RencodeValue::List(vec![]);

        let result: GlobResult = GlobResult::deserialize(&value).expect("deserialize");

        assert!(result.is_empty());
    }
}
