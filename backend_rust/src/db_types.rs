use rust_decimal::Decimal;
use std::str::FromStr;

pub fn str_to_decimal(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap_or(Decimal::ZERO)
}

pub fn decimal_to_str(d: &Decimal) -> String {
    d.to_string()
}

pub mod decimal_json {
    use rust_decimal::Decimal;
    use rust_decimal::prelude::ToPrimitive;
    use std::str::FromStr;
    use serde::{Serializer, Deserializer};

    pub fn serialize<S>(d: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(d.to_f64().unwrap_or(0.0))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        struct DecimalVisitor;

        impl<'de> Visitor<'de> for DecimalVisitor {
            type Value = Decimal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a number or string")
            }

            fn visit_f64<E>(self, v: f64) -> Result<Decimal, E>
            where
                E: de::Error,
            {
                Ok(Decimal::from_f64_retain(v).unwrap_or(Decimal::ZERO))
            }

            fn visit_str<E>(self, v: &str) -> Result<Decimal, E>
            where
                E: de::Error,
            {
                Decimal::from_str(v).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(DecimalVisitor)
    }
}