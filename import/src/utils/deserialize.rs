use std::fmt;
use std::num::ParseIntError;

use serde::Deserializer;
use serde::de::{Visitor, Error as DeserializeError};

use chrono::prelude::*;
use chrono::Duration;

use simulation::{Color, Direction};

pub fn numeric_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

pub fn naive_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
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
                .map_err(E::custom)
        }
    }

    deserializer.deserialize_str(NaiveDateVisitor)
}

pub fn duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
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
            let seconds = value.splitn(3, ':')
                .map(str::parse::<i64>)
                .try_fold(0, |acc, time| Ok(60 * acc + time?))
                .map_err(E::custom::<ParseIntError>);

            Ok(Duration::seconds(seconds?))
        }
    }

    deserializer.deserialize_str(DurationVisitor)
}

pub fn color<'de, D>(deserializer: D) -> Result<Color, D::Error>
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
                    .map_err(E::custom)
            };
            Ok(Color::new(component(0)?, component(1)?, component(2)?))
        }
    }

    deserializer.deserialize_str(ColorVisitor)
}

pub fn direction<'de, D>(deserializer: D) -> Result<Direction, D::Error>
    where D: Deserializer<'de>
{
    struct DirectionVisitor;

    impl<'de> Visitor<'de> for DirectionVisitor {
        type Value = Direction;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("either 0 or 1")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Direction, E>
            where E: DeserializeError
        {
            match value {
                0 => Ok(Direction::Upstream),
                1 => Ok(Direction::Downstream),
                _ => Err(E::custom(format!("invalid direction: {}", value))),
            }
        }
    }

    deserializer.deserialize_u64(DirectionVisitor)
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
        assert_eq!(numeric_bool(deserializer), Ok(true));
    }

    #[test]
    fn test_numeric_false() {
        let deserializer: U64Deserializer<ValueError> = 0u64.into_deserializer();
        assert_eq!(numeric_bool(deserializer), Ok(false));
    }

    #[test]
    fn test_numeric_invalid_number() {
        let deserializer: U64Deserializer<ValueError> = 2u64.into_deserializer();
        let error = numeric_bool(deserializer).unwrap_err();
        assert_eq!(error.description(), "invalid bool: 2");
    }

    #[test]
    fn test_numeric_empty() {
        let deserializer: StrDeserializer<ValueError> = "".into_deserializer();
        let error = numeric_bool(deserializer).unwrap_err();
        assert_eq!(error.description(), "invalid type: string \"\", expected either 0 or 1");
    }

    #[test]
    fn test_naive_date() {
        let deserializer: StrDeserializer<ValueError> = "20190711".into_deserializer();
        assert_eq!(naive_date(deserializer), Ok(NaiveDate::from_ymd(2019, 7, 11)));
    }

    #[test]
    fn test_naive_date_empty() {
        let deserializer: StrDeserializer<ValueError> = "".into_deserializer();
        let error = naive_date(deserializer).unwrap_err();
        assert_eq!(error.description(), "premature end of input");
    }

    #[test]
    fn test_naive_date_invalid_type() {
        let deserializer: U64Deserializer<ValueError> = 0u64.into_deserializer();
        let error = naive_date(deserializer).unwrap_err();
        assert_eq!(error.description(), "invalid type: integer `0`, expected date string");
    }

    fn from_hms(hours: i64, minutes: i64, seconds: i64) -> Duration {
        Duration::seconds((hours * 60 + minutes) * 60 + seconds)
    }

    #[test]
    fn test_duration_one_hour_digit() {
        let deserializer: StrDeserializer<ValueError> = "1:34:56".into_deserializer();
        assert_eq!(duration(deserializer), Ok(from_hms(1, 34, 56)));
    }

    #[test]
    fn test_duration_two_hour_digit() {
        let deserializer: StrDeserializer<ValueError> = "12:34:56".into_deserializer();
        assert_eq!(duration(deserializer), Ok(from_hms(12, 34, 56)));
    }

    #[test]
    fn test_duration_after_midnight() {
        let deserializer: StrDeserializer<ValueError> = "24:34:56".into_deserializer();
        assert_eq!(duration(deserializer), Ok(from_hms(24, 34, 56)));
    }

    #[test]
    fn test_duration_invalid_type() {
        let deserializer: U64Deserializer<ValueError> = 0u64.into_deserializer();
        let error = duration(deserializer).unwrap_err();
        assert_eq!(error.description(), "invalid type: integer `0`, expected time string");
    }

    #[test]
    fn test_color() {
        let deserializer: StrDeserializer<ValueError> = "#ff0420".into_deserializer();
        assert_eq!(color(deserializer), Ok(Color::new(255, 4, 32)));
    }

    #[test]
    fn test_direction_upstream() {
        let deserializer: U64Deserializer<ValueError> = 0u64.into_deserializer();
        assert_eq!(direction(deserializer), Ok(Direction::Upstream));
    }

    #[test]
    fn test_direction_downstream() {
        let deserializer: U64Deserializer<ValueError> = 1u64.into_deserializer();
        assert_eq!(direction(deserializer), Ok(Direction::Downstream));
    }

    #[test]
    fn test_invalid_direction() {
        let deserializer: U64Deserializer<ValueError> = 2u64.into_deserializer();
        let error = direction(deserializer).unwrap_err();
        assert_eq!(error.description(), "invalid direction: 2");
    }

    #[test]
    fn test_direction_empty() {
        let deserializer: StrDeserializer<ValueError> = "".into_deserializer();
        let error = direction(deserializer).unwrap_err();
        assert_eq!(error.description(), "invalid type: string \"\", expected either 0 or 1");
    }
}
