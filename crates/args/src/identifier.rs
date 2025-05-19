use alloc::{borrow::Cow, string::ToString};

use unicode_xid::UnicodeXID;

use crate::*;

/// IDENTIFIER_OR_KEYWORD
///
/// As defined in <https://doc.rust-lang.org/reference/identifiers.html>
///
/// Results in `format_args!("{match}", match = 10);` being a valid expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier<'a> {
    pub(crate) raw: bool,
    pub(crate) inner: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
#[error("failed to parse identifier")]
pub enum IdentifierParseError {
    #[error("first character--excluding any raw identifier marker ('r#')--may not begin with a underscore")]
    Underscore,
    #[error("‌Zero width unicode characters (U+200C and U+200D) aren't not allowed")]
    ZeroWidth,
    #[error("invalid XID_Start character")]
    InvalidStartCharacter,
    #[error("invalid XID_Continue character")]
    InvalidContinueCharacter,
}

impl<'a> Identifier<'a> {
    const RAW_START: &'static str = "r#";

    /// Context from `Argument::parse`:
    /// - `str` not empty
    pub(crate) fn parse(offset: usize, cow_str: impl Into<Cow<'a, str>>) -> Result<Self, ParseError> {
        let cow_str = cow_str.into();

        let (offset, ident, raw) = match cow_str.strip_prefix(Self::RAW_START) {
            Some(raw_ident) => (offset + Self::RAW_START.len(), raw_ident, true),
            None => (offset, cow_str.as_ref(), false),
        };

        assert_xid_chars(offset, ident)?;

        let inner = match cow_str {
            Cow::Borrowed(str) => {
                let str = match raw {
                    true => &str[Self::RAW_START.len()..],
                    false => str,
                };

                Cow::Borrowed(str)
            }
            Cow::Owned(mut string) => {
                let string = match raw {
                    true => string.split_off(Self::RAW_START.len()),
                    false => string,
                };

                Cow::Owned(string)
            }
        };

        return Ok(Identifier { raw, inner });

        fn assert_xid_chars(offset: usize, ident: &str) -> Result<(), ParseError> {
            const ZERO_WIDTH_NON_JOINER: char = '\u{200C}';
            const ZERO_WIDTH_JOINER: char = '\u{200D}';

            const UNDERSCORE: char = '_';

            let mut char_iter = ident.char_indices();

            let (_, first_char) = char_iter
                .next()
                .expect("call context should not have provided an empty string");

            {
                let first_char_error = if first_char == UNDERSCORE {
                    (ident.len() == 1).then_some(IdentifierParseError::Underscore)
                } else if first_char == ZERO_WIDTH_JOINER || first_char == ZERO_WIDTH_NON_JOINER {
                    Some(IdentifierParseError::ZeroWidth)
                } else if !first_char.is_xid_start() {
                    Some(IdentifierParseError::InvalidStartCharacter)
                } else {
                    None
                };

                if let Some(error) = first_char_error {
                    return Err(ParseError::new_char(offset, error));
                }
            }

            for (next_char_index, next_char) in char_iter {
                if !next_char.is_xid_continue() {
                    return single_char_error(offset, next_char_index, IdentifierParseError::InvalidContinueCharacter);
                } else if next_char == ZERO_WIDTH_JOINER || next_char == ZERO_WIDTH_NON_JOINER {
                    return single_char_error(offset, next_char_index, IdentifierParseError::ZeroWidth);
                }
            }

            Ok(())
        }

        fn single_char_error(
            offset: usize,
            char_index: usize,
            parse_error: IdentifierParseError,
        ) -> Result<(), ParseError> {
            Err(ParseError::new_char(offset + char_index, parse_error))
        }
    }

    pub(crate) fn owned(&self) -> Identifier<'static> {
        let Identifier { raw, inner } = self;
        Identifier { raw: *raw, inner: Cow::Owned(inner.to_string()) }
    }
}

#[cfg(feature = "serde")]
mod serde {
    use alloc::string::String;

    use ::serde::{Deserialize, Serialize, Serializer};

    use super::*;

    impl<'a> Serialize for Identifier<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self.raw {
                true => {
                    let mut ident_string = String::with_capacity(Self::RAW_START.len() + self.inner.len());
                    ident_string.push_str(Self::RAW_START);
                    ident_string.push_str(&self.inner);

                    serializer.serialize_str(&ident_string)
                }
                false => serializer.serialize_str(&self.inner),
            }
        }
    }

    impl<'a, 'de: 'a> Deserialize<'de> for Identifier<'a> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            let cow_str = Cow::<'a, str>::deserialize(deserializer)?;

            Self::parse(0, cow_str).map_err(|err| ::serde::de::Error::custom(err.kind()))
        }
    }

    #[cfg(test)]
    mod tests {
        use serde_json::Value;

        use super::*;

        #[test]
        fn serialize() {
            assert_serialize("r#x");
            assert_serialize("x_x");

            fn assert_serialize(ident_str: &str) {
                let ident = Identifier::parse(0, ident_str).unwrap();

                let expected_json = Value::String(ident_str.to_string());

                let actual_json = serde_json::to_value(ident).unwrap();

                assert_eq!(expected_json, actual_json)
            }
        }

        #[test]
        fn deserialize() {
            assert_deserialize("r#x");
            assert_deserialize("x_x");

            fn assert_deserialize(ident_str: &str) {
                let expected_ident = Identifier::parse(0, ident_str).unwrap();

                let value = Value::String(ident_str.to_string());
                let actual_ident = Identifier::deserialize(value).unwrap();

                assert_eq!(expected_ident, actual_ident)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::ops::Range;

    use super::*;

    #[test]
    fn parse() {
        assert_ok("x", false);
        assert_ok("r#x", true);
    }

    #[test]
    fn invalid_identifier_chars() {
        assert_error("1", 0..1, IdentifierParseError::InvalidStartCharacter);
        assert_error("r#1", 2..3, IdentifierParseError::InvalidStartCharacter);

        assert_error("x#", 1..2, IdentifierParseError::InvalidContinueCharacter);
        assert_error("r#x#", 3..4, IdentifierParseError::InvalidContinueCharacter);

        assert_error("\nx", 0..1, IdentifierParseError::InvalidStartCharacter);
        assert_error("x\n", 1..2, IdentifierParseError::InvalidContinueCharacter);
    }

    #[test]
    fn no_zero_width_joiner_error() {
        assert_error("\u{200D}x", 0..1, IdentifierParseError::ZeroWidth);
        assert_error("x\u{200D}", 1..2, IdentifierParseError::ZeroWidth);
    }

    #[test]
    fn no_zero_width_non_joiner_error() {
        assert_error("\u{200C}x", 0..1, IdentifierParseError::ZeroWidth);
        assert_error("x\u{200C}", 1..2, IdentifierParseError::ZeroWidth);
    }

    #[test]
    fn underscore_ok() {
        assert_ok("_x", false);
        assert_ok("r#_x", true);
    }

    #[test]
    fn underscore_error() {
        assert_error("_", 0..1, IdentifierParseError::Underscore);
        assert_error("r#_", 2..3, IdentifierParseError::Underscore);
    }

    fn assert_ok(ident_str: &str, expected_raw: bool) {
        let actual_ident = Identifier::parse(0, ident_str).unwrap();

        assert_eq!(expected_raw, actual_ident.raw);

        match expected_raw {
            true => assert_eq!(&ident_str[2..], actual_ident.inner),
            false => assert_eq!(ident_str, actual_ident.inner),
        }
    }

    fn assert_error(str: &str, expected_range: Range<usize>, expected_kind: IdentifierParseError) {
        let mock_offset = 10;

        let expected_error = ParseError::new(mock_offset, expected_range, expected_kind);

        let actual_error = Identifier::parse(mock_offset, str).unwrap_err();

        assert_eq!(expected_error, actual_error, "input string: {str}");
    }
}
