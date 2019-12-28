use std::fmt;

use serde::de::{Deserialize, Deserializer, Visitor, Error as DeserializeError};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ExceptionType {
    Added,
    Removed,
}

impl<'de> Deserialize<'de> for ExceptionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct ServiceExceptionTypeVisitor;

        impl<'de> Visitor<'de> for ServiceExceptionTypeVisitor {
            type Value = ExceptionType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("either 1 or 2")
            }

            fn visit_u64<E>(self, value: u64) -> Result<ExceptionType, E>
                where E: DeserializeError
            {
                match value {
                    1 => Ok(ExceptionType::Added),
                    2 => Ok(ExceptionType::Removed),
                    _ => Err(E::custom(format!("unknown exception type of value: {}", value))),
                }
            }
        }

        deserializer.deserialize_u64(ServiceExceptionTypeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error};

    #[test]
    fn test_deserialize_exception_type() {
        assert_de_tokens(&ExceptionType::Added, &[Token::U8(1)]);
        assert_de_tokens(&ExceptionType::Removed, &[Token::U8(2)]);
        assert_de_tokens_error::<ExceptionType>(&[Token::U8(0)],
                                                "unknown exception type of value: 0");
        assert_de_tokens_error::<ExceptionType>(&[Token::Str("")],
                                                "invalid type: string \"\", expected either 1 or 2");
    }
}
