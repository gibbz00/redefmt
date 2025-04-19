use alloc::string::{String, ToString};

use crate::*;

/// IDENTIFIER_OR_KEYWORD
///
/// As defined in <https://doc.rust-lang.org/reference/identifiers.html>
///
/// Results in `format_args!("{match}", match = 10);` being a valid expression.
#[derive(Debug, PartialEq)]
pub struct Identifier {
    raw: bool,
    // IMPROVEMENT: borrowed?
    inner: String,
}

#[derive(Debug, PartialEq)]
pub enum IdentifierParseError {
    /// ""
    Empty,
    /// "_"
    Underscore,
    /// "r#_"
    ReservedRaw,
    /// "r#<int>" or "<int>"
    LeadingInteger,
    /// "\t", " ", "\n" etc.
    WhiteSpace,
    /// "\u{200C}"
    ZeroWidthNonJoiner,
}

impl Parse for Identifier {
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        const RAW_START: &str = "r#";
        const UNDERSCORE: &str = "_";

        assert_not_empty(offset, str)?;

        assert_no_whitespace(offset, str)?;

        return match str.strip_prefix(RAW_START) {
            Some(raw_ident) => parse_raw(offset + RAW_START.len(), raw_ident),
            None => parse_non_raw(offset, str),
        };

        fn parse_raw(offset: usize, raw_ident: &str) -> Result<Identifier, ParseError> {
            if raw_ident.len() == 1 && raw_ident == UNDERSCORE {
                return Err(ParseError::new(offset, 0..1, IdentifierParseError::ReservedRaw));
            }

            assert_no_integer_start(offset, raw_ident)?;

            Ok(Identifier { raw: true, inner: raw_ident.to_string() })
        }

        fn parse_non_raw(offset: usize, ident: &str) -> Result<Identifier, ParseError> {
            if ident.len() == 1 && ident == UNDERSCORE {
                return Err(ParseError::new(offset, 0..1, IdentifierParseError::Underscore));
            }

            assert_no_integer_start(offset, ident)?;

            Ok(Identifier { raw: false, inner: ident.to_string() })
        }

        fn assert_not_empty(offset: usize, str: &str) -> Result<(), ParseError> {
            if str.is_empty() {
                return Err(ParseError::new(offset, 0..0, IdentifierParseError::Empty));
            }

            Ok(())
        }

        fn assert_no_whitespace(offset: usize, ident: &str) -> Result<(), ParseError> {
            const ZERO_WIDTH_NON_JOINER: char = '\u{200C}';

            for (ident_index, ident_char) in ident.chars().enumerate() {
                if ident_char.is_whitespace() {
                    return single_char_error(offset, ident_index, IdentifierParseError::WhiteSpace);
                }

                if ident_char == ZERO_WIDTH_NON_JOINER {
                    return single_char_error(offset, ident_index, IdentifierParseError::ZeroWidthNonJoiner);
                }
            }

            Ok(())
        }

        fn assert_no_integer_start(offset: usize, ident: &str) -> Result<(), ParseError> {
            ident
                .starts_with(|ch: char| !ch.is_ascii_digit())
                .then_some(())
                .ok_or(ParseError::new(offset, 0..1, IdentifierParseError::LeadingInteger))
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
    fn empty_error() {
        assert_error("", 0..0, IdentifierParseError::Empty);
    }

    #[test]
    fn whitespace_error() {
        assert_error("\nx", 0..1, IdentifierParseError::WhiteSpace);
        assert_error("x\n", 1..2, IdentifierParseError::WhiteSpace);
    }

    #[test]
    fn no_zero_width_non_joiner_error() {
        assert_error("\u{200C}x", 0..1, IdentifierParseError::ZeroWidthNonJoiner);
        assert_error("x\u{200C}", 1..2, IdentifierParseError::ZeroWidthNonJoiner);
    }

    #[test]
    fn raw_with_underscore_ok() {
        assert_ok("r#_x", true);
    }

    #[test]
    fn raw_with_underscore_error() {
        assert_error("r#_", 2..3, IdentifierParseError::ReservedRaw);
    }

    #[test]
    fn underscore_ok() {
        assert_ok("_x", false);
    }

    #[test]
    fn underscore_error() {
        assert_error("_", 0..1, IdentifierParseError::Underscore);
    }

    #[test]
    fn leading_integer_error() {
        assert_error("1", 0..1, IdentifierParseError::LeadingInteger);
        assert_error("r#1", 2..3, IdentifierParseError::LeadingInteger);
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

        assert_eq!(expected_error, actual_error);
    }
}
