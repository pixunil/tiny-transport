use std::fmt;

use serde::de::{Error as DeserializeError, Visitor};
use serde::Deserializer;

use chrono::Duration;

struct DurationVisitor;

impl<'de> Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("time string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Duration, E>
    where
        E: DeserializeError,
    {
        let seconds = value
            .splitn(3, ':')
            .map(|chunk| {
                str::parse::<i64>(chunk).map_err(|_| {
                    E::custom(format_args!(
                        "invalid time string: {}, invalid digit in {}",
                        value, chunk
                    ))
                })
            })
            .try_fold(0, |acc, time| Ok(60 * acc + time?));

        Ok(Duration::seconds(seconds?))
    }
}

pub(crate) fn duration<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    deserializer.deserialize_str(DurationVisitor)
}

#[cfg(test)]
mod tests {
    use serde::de::value::{Error as ValueError, StrDeserializer, U64Deserializer};
    use serde::de::IntoDeserializer;

    use super::*;

    fn from_hms(hours: i64, minutes: i64, seconds: i64) -> Duration {
        Duration::seconds((hours * 60 + minutes) * 60 + seconds)
    }

    #[test]
    fn test_one_hour_digit() {
        let deserializer: StrDeserializer<ValueError> = "1:34:56".into_deserializer();
        assert_eq!(duration(deserializer), Ok(from_hms(1, 34, 56)));
    }

    #[test]
    fn test_two_hour_digit() {
        let deserializer: StrDeserializer<ValueError> = "12:34:56".into_deserializer();
        assert_eq!(duration(deserializer), Ok(from_hms(12, 34, 56)));
    }

    #[test]
    fn test_after_midnight() {
        let deserializer: StrDeserializer<ValueError> = "24:34:56".into_deserializer();
        assert_eq!(duration(deserializer), Ok(from_hms(24, 34, 56)));
    }

    #[test]
    fn test_invalid_digit() {
        let deserializer: StrDeserializer<ValueError> = "12:IV:56".into_deserializer();
        assert_eq!(
            duration(deserializer).unwrap_err().to_string(),
            "invalid time string: 12:IV:56, invalid digit in IV"
        );
    }

    #[test]
    fn test_invalid_type() {
        let deserializer: U64Deserializer<ValueError> = 0u64.into_deserializer();
        assert_eq!(
            duration(deserializer).unwrap_err().to_string(),
            "invalid type: integer `0`, expected time string"
        );
    }
}
