use std::fmt;

use serde::de::{Error as DeserializeError, Visitor};
use serde::Deserializer;

use simulation::Color;

struct ColorVisitor;

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("color hex string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Color, E>
    where
        E: DeserializeError,
    {
        if value.len() != 7 {
            return Err(E::custom(format_args!(
                "invalid hex string: {}, expected 7 instead of {} characters",
                value,
                value.len()
            )));
        }

        let component = |number: usize| {
            let slice = &value[2 * number + 1..=2 * number + 2];
            u8::from_str_radix(slice, 16).map_err(|_| {
                E::custom(format_args!(
                    "invalid hex string: {}, invalid digit in {}",
                    value, slice
                ))
            })
        };
        Ok(Color::new(component(0)?, component(1)?, component(2)?))
    }
}

pub(crate) fn color<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Color, D::Error> {
    deserializer.deserialize_str(ColorVisitor)
}

#[cfg(test)]
mod tests {
    use serde::de::value::{Error as ValueError, StrDeserializer, U64Deserializer};
    use serde::de::IntoDeserializer;

    use super::*;

    #[test]
    fn test_six_digit_hex() {
        let deserializer: StrDeserializer<ValueError> = "#ff0420".into_deserializer();
        assert_eq!(color(deserializer), Ok(Color::new(255, 4, 32)));
    }

    #[test]
    fn test_invalid_digit() {
        let deserializer: StrDeserializer<ValueError> = "#12345g".into_deserializer();
        assert_eq!(
            color(deserializer).unwrap_err().to_string(),
            "invalid hex string: #12345g, invalid digit in 5g"
        );
    }

    #[test]
    fn test_too_few_digits() {
        let deserializer: StrDeserializer<ValueError> = "#12".into_deserializer();
        assert_eq!(
            color(deserializer).unwrap_err().to_string(),
            "invalid hex string: #12, expected 7 instead of 3 characters"
        );
    }

    #[test]
    fn test_too_many_digits() {
        let deserializer: StrDeserializer<ValueError> = "#1234567".into_deserializer();
        assert_eq!(
            color(deserializer).unwrap_err().to_string(),
            "invalid hex string: #1234567, expected 7 instead of 8 characters"
        );
    }

    #[test]
    fn test_empty() {
        let deserializer: StrDeserializer<ValueError> = "".into_deserializer();
        assert_eq!(
            color(deserializer).unwrap_err().to_string(),
            "invalid hex string: , expected 7 instead of 0 characters"
        );
    }

    #[test]
    fn test_invalid_type() {
        let deserializer: U64Deserializer<ValueError> = 0u64.into_deserializer();
        assert_eq!(
            color(deserializer).unwrap_err().to_string(),
            "invalid type: integer `0`, expected color hex string"
        );
    }
}
