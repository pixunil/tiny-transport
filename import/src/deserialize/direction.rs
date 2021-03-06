use std::fmt;

use serde::de::{Error as DeserializeError, Unexpected, Visitor};
use serde::Deserializer;

use simulation::Direction;

struct DirectionVisitor;

impl<'de> Visitor<'de> for DirectionVisitor {
    type Value = Direction;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("either 0 or 1")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Direction, E>
    where
        E: DeserializeError,
    {
        match value {
            0 => Ok(Direction::Upstream),
            1 => Ok(Direction::Downstream),
            _ => Err(E::invalid_value(Unexpected::Unsigned(value), &self)),
        }
    }
}

pub(crate) fn direction<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Direction, D::Error> {
    deserializer.deserialize_u64(DirectionVisitor)
}

#[cfg(test)]
mod tests {
    use serde::de::value::{Error as ValueError, StrDeserializer, U64Deserializer};
    use serde::de::IntoDeserializer;

    use super::*;

    #[test]
    fn test_upstream() {
        let deserializer: U64Deserializer<ValueError> = 0u64.into_deserializer();
        assert_eq!(direction(deserializer), Ok(Direction::Upstream));
    }

    #[test]
    fn test_downstream() {
        let deserializer: U64Deserializer<ValueError> = 1u64.into_deserializer();
        assert_eq!(direction(deserializer), Ok(Direction::Downstream));
    }

    #[test]
    fn test_invalid() {
        let deserializer: U64Deserializer<ValueError> = 2u64.into_deserializer();
        assert_eq!(
            direction(deserializer).unwrap_err().to_string(),
            "invalid value: integer `2`, expected either 0 or 1"
        );
    }

    #[test]
    fn test_empty() {
        let deserializer: StrDeserializer<ValueError> = "".into_deserializer();
        assert_eq!(
            direction(deserializer).unwrap_err().to_string(),
            "invalid type: string \"\", expected either 0 or 1"
        );
    }
}
