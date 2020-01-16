use std::fmt;

use serde::Deserializer;
use serde::de::{Visitor, Error as DeserializeError};

use chrono::NaiveDate;

struct NaiveDateVisitor;

impl<'de> Visitor<'de> for NaiveDateVisitor {
    type Value = NaiveDate;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("date string")
    }

    fn visit_str<E>(self, value: &str) -> Result<NaiveDate, E>
        where E: DeserializeError
    {
        NaiveDate::parse_from_str(value, "%Y%m%d")
            .map_err(E::custom)
    }
}

pub(crate) fn naive_date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDate, D::Error> {
    deserializer.deserialize_str(NaiveDateVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::de::IntoDeserializer;
    use serde::de::value::{U64Deserializer, StrDeserializer, Error as ValueError};

    #[test]
    fn test_date() {
        let deserializer: StrDeserializer<ValueError> = "20190711".into_deserializer();
        assert_eq!(naive_date(deserializer), Ok(NaiveDate::from_ymd(2019, 7, 11)));
    }

    #[test]
    fn test_empty() {
        let deserializer: StrDeserializer<ValueError> = "".into_deserializer();
        let error = naive_date(deserializer).unwrap_err();
        assert_eq!(error.to_string(), "premature end of input");
    }

    #[test]
    fn test_invalid_type() {
        let deserializer: U64Deserializer<ValueError> = 0u64.into_deserializer();
        let error = naive_date(deserializer).unwrap_err();
        assert_eq!(error.to_string(), "invalid type: integer `0`, expected date string");
    }
}
