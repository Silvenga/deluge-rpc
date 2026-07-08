use serde::de::{self, Deserializer, Visitor};
use std::fmt;

/// Deserialize an `Option<i64>` where `-1` represents unlimited (None).
pub fn deserialize_unlimited_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct UnlimitedI64Visitor;

    impl<'de> Visitor<'de> for UnlimitedI64Visitor {
        type Value = Option<i64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer, where -1 means unlimited")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Option<i64>, E>
        where
            E: de::Error,
        {
            match value {
                -1 => Ok(None),
                other => Ok(Some(other)),
            }
        }
    }

    deserializer.deserialize_i64(UnlimitedI64Visitor)
}

/// Deserialize an `Option<f64>` where `-1.0` represents unlimited (None).
pub fn deserialize_unlimited_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct UnlimitedF64Visitor;

    impl<'de> Visitor<'de> for UnlimitedF64Visitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a float, where -1.0 means unlimited")
        }

        fn visit_f64<E>(self, value: f64) -> Result<Option<f64>, E>
        where
            E: de::Error,
        {
            if value == -1.0 {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }

        fn visit_i64<E>(self, value: i64) -> Result<Option<f64>, E>
        where
            E: de::Error,
        {
            if value == -1 {
                Ok(None)
            } else {
                Ok(Some(value as f64))
            }
        }
    }

    deserializer.deserialize_f64(UnlimitedF64Visitor)
}

/// Deserialize an `Option<i64>` where `-1` represents never (None).
///
/// Used for timestamps like `completed_time`, `time_since_download`, etc.
pub fn deserialize_never_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct NeverI64Visitor;

    impl<'de> Visitor<'de> for NeverI64Visitor {
        type Value = Option<i64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer, where -1 means never")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Option<i64>, E>
        where
            E: de::Error,
        {
            match value {
                -1 => Ok(None),
                other => Ok(Some(other)),
            }
        }
    }

    deserializer.deserialize_i64(NeverI64Visitor)
}

/// Deserialize an `Option<f64>` where `-1.0` represents infinity (None).
///
/// Used for ratio fields where `-1.0` means infinity (when `total_done == 0`).
pub fn deserialize_ratio<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct RatioVisitor;

    impl<'de> Visitor<'de> for RatioVisitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a float, where -1.0 means infinity")
        }

        fn visit_f64<E>(self, value: f64) -> Result<Option<f64>, E>
        where
            E: de::Error,
        {
            if value == -1.0 {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }

        fn visit_i64<E>(self, value: i64) -> Result<Option<f64>, E>
        where
            E: de::Error,
        {
            if value == -1 {
                Ok(None)
            } else {
                Ok(Some(value as f64))
            }
        }
    }

    deserializer.deserialize_f64(RatioVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deluge_rpc_rencode::RencodeValue;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    fn make_dict(key: &str, value: RencodeValue) -> RencodeValue {
        let mut map = BTreeMap::new();
        map.insert(RencodeValue::Str(key.into()), value);
        RencodeValue::Dict(map)
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct UnlimitedI64 {
        #[serde(deserialize_with = "deserialize_unlimited_i64")]
        field: Option<i64>,
    }

    #[test]
    fn when_int_is_minus_one_then_unlimited_i64_returns_none() {
        let value = make_dict("field", RencodeValue::Int(-1));

        let result: UnlimitedI64 = UnlimitedI64::deserialize(&value).expect("deserialize");

        assert_eq!(result, UnlimitedI64 { field: None });
    }

    #[test]
    fn when_int_is_positive_then_unlimited_i64_returns_some() {
        let value = make_dict("field", RencodeValue::Int(42));

        let result: UnlimitedI64 = UnlimitedI64::deserialize(&value).expect("deserialize");

        assert_eq!(result, UnlimitedI64 { field: Some(42) });
    }

    #[test]
    fn when_int_is_zero_then_unlimited_i64_returns_some() {
        let value = make_dict("field", RencodeValue::Int(0));

        let result: UnlimitedI64 = UnlimitedI64::deserialize(&value).expect("deserialize");

        assert_eq!(result, UnlimitedI64 { field: Some(0) });
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct UnlimitedF64 {
        #[serde(deserialize_with = "deserialize_unlimited_f64")]
        field: Option<f64>,
    }

    #[test]
    fn when_float_is_minus_one_then_unlimited_f64_returns_none() {
        let value = make_dict("field", RencodeValue::Float(-1.0));

        let result: UnlimitedF64 = UnlimitedF64::deserialize(&value).expect("deserialize");

        assert_eq!(result, UnlimitedF64 { field: None });
    }

    #[test]
    fn when_float_is_positive_then_unlimited_f64_returns_some() {
        let value = make_dict("field", RencodeValue::Float(500.0));

        let result: UnlimitedF64 = UnlimitedF64::deserialize(&value).expect("deserialize");

        assert_eq!(result, UnlimitedF64 { field: Some(500.0) });
    }

    #[test]
    fn when_float_is_zero_then_unlimited_f64_returns_some() {
        let value = make_dict("field", RencodeValue::Float(0.0));

        let result: UnlimitedF64 = UnlimitedF64::deserialize(&value).expect("deserialize");

        assert_eq!(result, UnlimitedF64 { field: Some(0.0) });
    }

    #[test]
    fn when_unlimited_f64_receives_int_then_converts() {
        let value = make_dict("field", RencodeValue::Int(500));

        let result: UnlimitedF64 = UnlimitedF64::deserialize(&value).expect("deserialize");

        assert_eq!(result, UnlimitedF64 { field: Some(500.0) });
    }

    #[test]
    fn when_unlimited_f64_receives_int_minus_one_then_none() {
        let value = make_dict("field", RencodeValue::Int(-1));

        let result: UnlimitedF64 = UnlimitedF64::deserialize(&value).expect("deserialize");

        assert_eq!(result, UnlimitedF64 { field: None });
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct NeverI64 {
        #[serde(deserialize_with = "deserialize_never_i64")]
        field: Option<i64>,
    }

    #[test]
    fn when_never_timestamp_is_minus_one_then_never_returns_none() {
        let value = make_dict("field", RencodeValue::Int(-1));

        let result: NeverI64 = NeverI64::deserialize(&value).expect("deserialize");

        assert_eq!(result, NeverI64 { field: None });
    }

    #[test]
    fn when_never_timestamp_is_positive_then_never_returns_some() {
        let value = make_dict("field", RencodeValue::Int(1_700_000_000));

        let result: NeverI64 = NeverI64::deserialize(&value).expect("deserialize");

        assert_eq!(
            result,
            NeverI64 {
                field: Some(1_700_000_000)
            }
        );
    }

    #[test]
    fn when_never_timestamp_is_zero_then_never_returns_some() {
        let value = make_dict("field", RencodeValue::Int(0));

        let result: NeverI64 = NeverI64::deserialize(&value).expect("deserialize");

        assert_eq!(result, NeverI64 { field: Some(0) });
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Ratio {
        #[serde(deserialize_with = "deserialize_ratio")]
        field: Option<f64>,
    }

    #[test]
    fn when_ratio_is_minus_one_then_ratio_returns_none() {
        let value = make_dict("field", RencodeValue::Float(-1.0));

        let result: Ratio = Ratio::deserialize(&value).expect("deserialize");

        assert_eq!(result, Ratio { field: None });
    }

    #[test]
    fn when_ratio_is_positive_then_ratio_returns_some() {
        let value = make_dict("field", RencodeValue::Float(2.5));

        let result: Ratio = Ratio::deserialize(&value).expect("deserialize");

        assert_eq!(result, Ratio { field: Some(2.5) });
    }

    #[test]
    fn when_ratio_is_zero_then_ratio_returns_some() {
        let value = make_dict("field", RencodeValue::Float(0.0));

        let result: Ratio = Ratio::deserialize(&value).expect("deserialize");

        assert_eq!(result, Ratio { field: Some(0.0) });
    }

    #[test]
    fn when_ratio_receives_int_then_converts() {
        let value = make_dict("field", RencodeValue::Int(3));

        let result: Ratio = Ratio::deserialize(&value).expect("deserialize");

        assert_eq!(result, Ratio { field: Some(3.0) });
    }

    #[test]
    fn when_ratio_receives_int_minus_one_then_none() {
        let value = make_dict("field", RencodeValue::Int(-1));

        let result: Ratio = Ratio::deserialize(&value).expect("deserialize");

        assert_eq!(result, Ratio { field: None });
    }
}
