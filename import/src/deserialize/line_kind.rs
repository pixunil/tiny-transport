use std::fmt;

use serde::de::{Error as DeserializeError, Visitor};
use serde::Deserializer;

use simulation::line::Kind as LineKind;

struct LineKindVisitor;

impl<'de> Visitor<'de> for LineKindVisitor {
    type Value = LineKind;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("positive integer")
    }

    fn visit_u64<E>(self, value: u64) -> Result<LineKind, E>
    where
        E: DeserializeError,
    {
        match value {
            2 | 100 => Ok(LineKind::Railway),
            109 => Ok(LineKind::SuburbanRailway),
            1 | 400 => Ok(LineKind::UrbanRailway),
            3 | 700 => Ok(LineKind::Bus),
            0 | 900 => Ok(LineKind::Tram),
            4 | 1000 => Ok(LineKind::WaterTransport),
            _ => Err(E::custom(format!("unknown line kind of value: {}", value))),
        }
    }
}

pub(crate) fn line_kind<'de, D: Deserializer<'de>>(deserializer: D) -> Result<LineKind, D::Error> {
    deserializer.deserialize_u64(LineKindVisitor)
}

#[cfg(test)]
mod tests {
    use serde::de::value::{Error as ValueError, StrDeserializer, U64Deserializer};
    use serde::de::IntoDeserializer;

    use super::*;

    #[test]
    fn test_deserialize() {
        let deserializer: U64Deserializer<ValueError> = 2u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::Railway));
        let deserializer: U64Deserializer<ValueError> = 100u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::Railway));
        let deserializer: U64Deserializer<ValueError> = 109u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::SuburbanRailway));
        let deserializer: U64Deserializer<ValueError> = 1u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::UrbanRailway));
        let deserializer: U64Deserializer<ValueError> = 400u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::UrbanRailway));
        let deserializer: U64Deserializer<ValueError> = 3u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::Bus));
        let deserializer: U64Deserializer<ValueError> = 700u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::Bus));
        let deserializer: U64Deserializer<ValueError> = 0u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::Tram));
        let deserializer: U64Deserializer<ValueError> = 900u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::Tram));
        let deserializer: U64Deserializer<ValueError> = 4u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::WaterTransport));
        let deserializer: U64Deserializer<ValueError> = 1000u64.into_deserializer();
        assert_eq!(line_kind(deserializer), Ok(LineKind::WaterTransport));
    }

    #[test]
    fn test_unknown_line_kind() {
        let deserializer: U64Deserializer<ValueError> = 9999u64.into_deserializer();
        assert_eq!(
            line_kind(deserializer).unwrap_err().to_string(),
            "unknown line kind of value: 9999"
        );
    }

    #[test]
    fn test_empty() {
        let deserializer: StrDeserializer<ValueError> = "".into_deserializer();
        assert_eq!(
            line_kind(deserializer).unwrap_err().to_string(),
            "invalid type: string \"\", expected positive integer"
        );
    }
}
