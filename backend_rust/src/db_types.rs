use rust_decimal::Decimal;
use std::str::FromStr;

pub fn parse_decimal(s: &str) -> Result<Decimal, rust_decimal::Error> {
    Decimal::from_str(s)
}

pub fn str_to_decimal(s: &str) -> Decimal {
    match parse_decimal(s) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("WARN: invalid decimal string '{s}': {e}");
            Decimal::ZERO
        }
    }
}

pub fn decimal_to_str(d: &Decimal) -> String {
    d.to_string()
}

pub mod decimal_json {
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(d: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&d.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DecimalVisitor;

        impl<'de> serde::de::Visitor<'de> for DecimalVisitor {
            type Value = Decimal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or number")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Decimal, E> {
                super::parse_decimal(v).map_err(|_| E::custom(format!("invalid decimal: {v}")))
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Decimal, E> {
                Decimal::from_i64(v).ok_or_else(|| E::custom("i64 does not fit decimal"))
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Decimal, E> {
                Decimal::from_u64(v).ok_or_else(|| E::custom("u64 does not fit decimal"))
            }

            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Decimal, E> {
                // Lenient fallback for legacy numeric clients; inherently lossy.
                Decimal::from_f64_retain(v).ok_or_else(|| E::custom("f64 does not fit decimal"))
            }
        }

        deserializer.deserialize_any(DecimalVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Wrap { #[serde(with = "decimal_json")] d: Decimal }

    #[test]
    fn serialize_is_exact_string() {
        let w = Wrap { d: Decimal::from_str("9.99").unwrap() };
        assert_eq!(serde_json::to_string(&w).unwrap(), r#"{"d":"9.99"}"#);
    }

    #[test]
    fn integer_input_is_exact() {
        let w: Wrap = serde_json::from_str(r#"{"d":100}"#).unwrap();
        assert_eq!(w.d, Decimal::from_str("100").unwrap());
        let w: Wrap = serde_json::from_str(r#"{"d":-7}"#).unwrap();
        assert_eq!(w.d, Decimal::from_str("-7").unwrap());
    }

    #[test]
    fn string_input_is_exact() {
        let w: Wrap = serde_json::from_str(r#"{"d":"9.99"}"#).unwrap();
        assert_eq!(w.d, Decimal::from_str("9.99").unwrap());
    }

    #[test]
    fn fractional_sum_is_exact() {
        let sum = Decimal::from_str("0.1").unwrap() + Decimal::from_str("0.2").unwrap();
        assert_eq!(sum, Decimal::from_str("0.3").unwrap());
        assert_eq!(serde_json::to_string(&Wrap { d: sum }).unwrap(), r#"{"d":"0.3"}"#);
    }

    #[test]
    fn invalid_string_input_errors() {
        let res = serde_json::from_str::<Wrap>(r#"{"d":"abc"}"#);
        assert!(res.is_err());
    }
}