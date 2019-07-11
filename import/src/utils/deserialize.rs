use std::fmt;

use serde::Deserializer;
use serde::de::{Visitor, Error as DeserializeError};

use chrono::prelude::*;
use chrono::Duration;

use simulation::Color;

pub fn deserialize_numeric_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where D: Deserializer<'de>
{
    struct NumericBoolVisitor;

    impl<'de> Visitor<'de> for NumericBoolVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("either 0 or 1")
        }

        fn visit_u64<E>(self, value: u64) -> Result<bool, E>
            where E: DeserializeError
        {
            match value {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(E::custom(format!("invalid bool: {}", value))),
            }
        }
    }

    deserializer.deserialize_u64(NumericBoolVisitor)
}

pub fn deserialize_naive_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where D: Deserializer<'de>
{
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
                .map_err(|err| E::custom(err))
        }
    }

    deserializer.deserialize_str(NaiveDateVisitor)
}

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where D: Deserializer<'de>
{
    struct DurationVisitor;

    impl<'de> Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("time string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Duration, E>
            where E: DeserializeError
        {
            let seconds = value.split(":")
                .map(|part| part.parse::<i64>())
                .try_fold(0, |acc, time| Ok(60 * acc + time?))
                .map_err(|err: ::std::num::ParseIntError| E::custom(err));

            Ok(Duration::seconds(seconds?))
        }
    }

    deserializer.deserialize_str(DurationVisitor)
}

pub fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where D: Deserializer<'de>
{
    struct ColorVisitor;

    impl<'de> Visitor<'de> for ColorVisitor {
        type Value = Color;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("color hex string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Color, E>
            where E: DeserializeError
        {
            let component = |number: usize| {
                let slice = value.get(2 * number + 1 ..= 2 * number + 2)
                    .ok_or_else(|| E::custom(""))?;
                u8::from_str_radix(slice, 16)
                    .map_err(|err| E::custom(err))
            };
            Ok(Color::new(component(0)?, component(1)?, component(2)?))
        }
    }

    deserializer.deserialize_str(ColorVisitor)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use serde::de::IntoDeserializer;
    use serde::de::value::{U64Deserializer, StrDeserializer, Error as ValueError};

    use super::*;

    #[test]
    fn test_numeric_true() {
        let deserializer: U64Deserializer<ValueError> = 1u64.into_deserializer();
        assert_eq!(deserialize_numeric_bool(deserializer), Ok(true));
    }

    #[test]
    fn test_numeric_false() {
        let deserializer: U64Deserializer<ValueError> = 0u64.into_deserializer();
        assert_eq!(deserialize_numeric_bool(deserializer), Ok(false));
    }

    #[test]
    fn test_numeric_invalid_number() {
        let deserializer: U64Deserializer<ValueError> = 2u64.into_deserializer();
        let error = deserialize_numeric_bool(deserializer).unwrap_err();
        assert_eq!(error.description(), "invalid bool: 2");
    }

    #[test]
    fn test_numeric_empty() {
        let deserializer: StrDeserializer<ValueError> = "".into_deserializer();
        let error = deserialize_numeric_bool(deserializer).unwrap_err();
        assert_eq!(error.description(), "invalid type: string \"\", expected either 0 or 1");
    }

    #[test]
    fn test_naive_date() {
        let deserializer: StrDeserializer<ValueError> = "20190711".into_deserializer();
        assert_eq!(deserialize_naive_date(deserializer), Ok(NaiveDate::from_ymd(2019, 7, 11)));
    }

    fn duration(hours: i64, minutes: i64, seconds: i64) -> Duration {
        Duration::seconds((hours * 60 + minutes) * 60 + seconds)
    }

    #[test]
    fn test_duration_one_hour_digit() {
        let deserializer: StrDeserializer<ValueError> = "1:34:56".into_deserializer();
        assert_eq!(deserialize_duration(deserializer), Ok(duration(1, 34, 56)));
    }

    #[test]
    fn test_duration_two_hour_digit() {
        let deserializer: StrDeserializer<ValueError> = "12:34:56".into_deserializer();
        assert_eq!(deserialize_duration(deserializer), Ok(duration(12, 34, 56)));
    }

    #[test]
    fn test_duration_after_midnight() {
        let deserializer: StrDeserializer<ValueError> = "24:34:56".into_deserializer();
        assert_eq!(deserialize_duration(deserializer), Ok(duration(24, 34, 56)));
    }

    #[test]
    fn test_color() {
        let deserializer: StrDeserializer<ValueError> = "#ff0420".into_deserializer();
        assert_eq!(deserialize_color(deserializer), Ok(Color::new(255, 4, 32)));
    }
}
