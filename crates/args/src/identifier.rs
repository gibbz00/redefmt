use alloc::string::{String, ToString};

use unicode_xid::UnicodeXID;

use crate::*;

/// IDENTIFIER_OR_KEYWORD
///
/// As defined in <https://doc.rust-lang.org/reference/identifiers.html>
///
/// Results in `format_args!("{match}", match = 10);` being a valid expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    raw: bool,
    // IMPROVEMENT: borrowed?
    inner: String,
}

#[derive(Debug, PartialEq)]
pub enum IdentifierParseError {
    /// "_" or "r#_"
    Underscore,
    /// "\u{200C}"
    ZeroWidthNonJoiner,
    InvalidStartCharacter,
    InvalidContinueCharacter,
}

impl Parse for Identifier {
    // Context from `Argument::parse`:
    // - `str` not empty
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        const RAW_START: &str = "r#";

        let (offset, ident, raw) = match str.strip_prefix(RAW_START) {
            Some(raw_ident) => (offset + RAW_START.len(), raw_ident, true),
            None => (offset, str, false),
        };

        assert_xid_chars(offset, ident)?;

        return Ok(Identifier { raw, inner: ident.to_string() });

        fn assert_xid_chars(offset: usize, ident: &str) -> Result<(), ParseError> {
            const ZERO_WIDTH_NON_JOINER: char = '\u{200C}';
            const UNDERSCORE: char = '_';

            let mut char_iter = ident.chars().enumerate();

            let (_, first_char) = char_iter
                .next()
                .expect("call context should not have provided an empty string");

            if first_char == UNDERSCORE {
                if ident.len() == 1 {
                    return Err(ParseError::new(offset, 0..1, IdentifierParseError::Underscore));
                }
            } else if first_char == ZERO_WIDTH_NON_JOINER {
                return single_char_error(offset, 0, IdentifierParseError::ZeroWidthNonJoiner);
            } else if !first_char.is_xid_start() {
                return single_char_error(offset, 0, IdentifierParseError::InvalidStartCharacter);
            }

            for (next_char_index, next_char) in char_iter {
                if !next_char.is_xid_continue() {
                    return single_char_error(offset, next_char_index, IdentifierParseError::InvalidContinueCharacter);
                } else if next_char == ZERO_WIDTH_NON_JOINER {
                    return single_char_error(offset, next_char_index, IdentifierParseError::ZeroWidthNonJoiner);
                }
            }

            Ok(())
        }

        fn single_char_error(
            offset: usize,
            char_index: usize,
            parse_error: IdentifierParseError,
        ) -> Result<(), ParseError> {
            Err(ParseError::new(offset + char_index, 0..1, parse_error))
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
    fn no_zero_width_non_joiner_error() {
        assert_error("\u{200C}x", 0..1, IdentifierParseError::ZeroWidthNonJoiner);
        assert_error("x\u{200C}", 1..2, IdentifierParseError::ZeroWidthNonJoiner);
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
            true => assert_eq!(&ident_str[2..], &actual_ident.inner),
            false => assert_eq!(ident_str, &actual_ident.inner),
        }
    }

    fn assert_error(str: &str, expected_range: Range<usize>, expected_kind: IdentifierParseError) {
        let mock_offset = 10;

        let expected_error = ParseError::new(mock_offset, expected_range, expected_kind);

        let actual_error = Identifier::parse(mock_offset, str).unwrap_err();

        assert_eq!(expected_error, actual_error, "input string: {}", str);
    }
}
