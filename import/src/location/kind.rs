use std::fmt;

use serde::de::{Deserialize, Deserializer, Visitor, Error as DeserializeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum LocationKind {
    Stop,
    Station,
    Entrance,
    GenericNode,
    BoardingArea,
}

impl<'de> Deserialize<'de> for LocationKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct LineKindVisitor;

        impl<'de> Visitor<'de> for LineKindVisitor {
            type Value = LocationKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("integer")
            }

            fn visit_u64<E>(self, value: u64) -> Result<LocationKind, E>
                where E: DeserializeError
            {
                match value {
                    0 => Ok(LocationKind::Stop),
                    1 => Ok(LocationKind::Station),
                    2 => Ok(LocationKind::Entrance),
                    3 => Ok(LocationKind::GenericNode),
                    4 => Ok(LocationKind::BoardingArea),
                    _ => Err(E::custom(format!("unknown location type of value: {}", value))),
                }
            }
        }

        deserializer.deserialize_u64(LineKindVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error};

    #[test]
    fn test_deserialize_location_kind() {
        assert_de_tokens(&LocationKind::Stop, &[Token::U16(0)]);
        assert_de_tokens(&LocationKind::Station, &[Token::U16(1)]);
        assert_de_tokens(&LocationKind::Entrance, &[Token::U16(2)]);
        assert_de_tokens(&LocationKind::GenericNode, &[Token::U16(3)]);
        assert_de_tokens(&LocationKind::BoardingArea, &[Token::U16(4)]);
        assert_de_tokens_error::<LocationKind>(&[Token::U16(5)],
                                               "unknown location type of value: 5");
        assert_de_tokens_error::<LocationKind>(&[Token::Str("")],
                                               "invalid type: string \"\", expected integer");
    }
}
