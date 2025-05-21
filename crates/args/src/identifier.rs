use alloc::{borrow::Cow, string::ToString};
use core::fmt::{Debug, Display};

use unicode_xid::UnicodeXID;

use crate::*;

/// Non-raw identifier or keyword
///
/// As defined in <https://doc.rust-lang.org/reference/identifiers.html>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier<'a> {
    inner: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
#[error("failed to parse identifier")]
pub enum IdentifierParseError {
    #[error("raw identifiers are not allowed in format strings")]
    RawIdent,
    #[error("first character may not begin with a underscore")]
    Underscore,
    #[error("‌Zero width unicode characters (U+200C and U+200D) aren't not allowed")]
    ZeroWidth,
    #[error("invalid XID_Start character")]
    InvalidStartCharacter,
    #[error("invalid XID_Continue character")]
    InvalidContinueCharacter,
}

#[cfg(feature = "provided-args")]
impl Identifier<'static> {
    pub fn from_provided(ident: &syn::Ident) -> Self {
        use syn::ext::IdentExt;
        Self { inner: ident.unraw().to_string().into() }
    }
}

impl<'a> Identifier<'a> {
    const RAW_START: &'static str = "r#";

    /// Context from `Argument::parse`:
    /// - `str` not empty
    pub(crate) fn parse(offset: usize, cow_str: impl Into<Cow<'a, str>>) -> Result<Self, ParseError> {
        let cow_str = cow_str.into();

        if cow_str.starts_with(Self::RAW_START) {
            return Err(ParseError::new(
                offset,
                0..Self::RAW_START.len(),
                IdentifierParseError::RawIdent,
            ));
        }

        assert_xid_chars(offset, &cow_str)?;

        return Ok(Identifier { inner: cow_str });

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
        let Identifier { inner } = self;
        Identifier { inner: Cow::Owned(inner.to_string()) }
    }
}

impl AsRef<str> for Identifier<'_> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Display for Identifier<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

#[cfg(feature = "serde")]
mod serde {
    use ::serde::{Deserialize, Serialize, Serializer};

    use super::*;

    impl<'a> Serialize for Identifier<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.inner)
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
            assert_serialize("x");

            fn assert_serialize(ident_str: &str) {
                let ident = Identifier::parse(0, ident_str).unwrap();

                let expected_json = Value::String(ident_str.to_string());

                let actual_json = serde_json::to_value(ident).unwrap();

                assert_eq!(expected_json, actual_json)
            }
        }

        #[test]
        fn deserialize() {
            assert_deserialize("x");

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
        assert_ok("x");
    }

    #[test]
    fn display() {
        assert_display("x");

        fn assert_display(expected: &str) {
            let actual = Identifier::parse(0, expected).unwrap().to_string();
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn invalid_identifier_chars() {
        assert_error("1", 0..1, IdentifierParseError::InvalidStartCharacter);
        assert_error("x#", 1..2, IdentifierParseError::InvalidContinueCharacter);
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
    fn raw_identifier_error() {
        assert_error("r#x", 0..2, IdentifierParseError::RawIdent);
    }

    #[test]
    fn underscore_ok() {
        assert_ok("_x");
    }

    #[test]
    fn underscore_error() {
        assert_error("_", 0..1, IdentifierParseError::Underscore);
    }

    fn assert_ok(ident_str: &str) {
        let actual_ident = Identifier::parse(0, ident_str).unwrap();
        assert_eq!(ident_str, actual_ident.inner)
    }

    fn assert_error(str: &str, expected_range: Range<usize>, expected_kind: IdentifierParseError) {
        let mock_offset = 10;

        let expected_error = ParseError::new(mock_offset, expected_range, expected_kind);

        let actual_error = Identifier::parse(mock_offset, str).unwrap_err();

        assert_eq!(expected_error, actual_error, "input string: {str}");
    }
}
