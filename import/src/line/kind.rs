use std::fmt;

use serde::de::{Deserialize, Deserializer, Visitor, Error as DeserializeError};

use simulation::Color;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Kind {
    Railway,
    SuburbanRailway,
    UrbanRailway,
    Bus,
    Tram,
    WaterTransport,
}

impl Kind {
    pub(super) fn color(self) -> Color {
        match self {
            Kind::Railway => Color::new(227, 0, 27),
            Kind::SuburbanRailway => Color::new(0, 114, 56),
            Kind::UrbanRailway => Color::new(0, 100, 173),
            Kind::Bus => Color::new(125, 23, 107),
            Kind::Tram => Color::new(204, 10, 34),
            Kind::WaterTransport => Color::new(0, 128, 186),
        }
    }
}

impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct LineKindVisitor;

        impl<'de> Visitor<'de> for LineKindVisitor {
            type Value = Kind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("positive integer")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Kind, E>
                where E: DeserializeError
            {
                match value {
                    100 => Ok(Kind::Railway),
                    109 => Ok(Kind::SuburbanRailway),
                    400 => Ok(Kind::UrbanRailway),
                    3 | 700 => Ok(Kind::Bus),
                    900 => Ok(Kind::Tram),
                    1000 => Ok(Kind::WaterTransport),
                    _ => Err(E::custom(format!("unknown route kind of value: {}", value))),
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
    fn test_deserialize_line_kind() {
        assert_de_tokens(&Kind::Railway, &[Token::U16(100)]);
        assert_de_tokens(&Kind::SuburbanRailway, &[Token::U16(109)]);
        assert_de_tokens(&Kind::UrbanRailway, &[Token::U16(400)]);
        assert_de_tokens(&Kind::Bus, &[Token::U16(3)]);
        assert_de_tokens(&Kind::Bus, &[Token::U16(700)]);
        assert_de_tokens(&Kind::Tram, &[Token::U16(900)]);
        assert_de_tokens(&Kind::WaterTransport, &[Token::U16(1000)]);
        assert_de_tokens_error::<Kind>(&[Token::U16(0)],
                                       "unknown route kind of value: 0");
        assert_de_tokens_error::<Kind>(&[Token::Str("")],
                                       "invalid type: string \"\", expected positive integer");
    }
}
