use unicode_xid::UnicodeXID;

use crate::*;

pub(crate) const RAW_START: &str = "r#";

pub(crate) fn assert_xid_chars(offset: usize, ident: &str) -> Result<(), ParseError> {
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

    return Ok(());

    fn single_char_error(
        offset: usize,
        char_index: usize,
        parse_error: IdentifierParseError,
    ) -> Result<(), ParseError> {
        Err(ParseError::new_char(offset + char_index, parse_error))
    }
}

#[cfg(test)]
mod tests {
    use core::ops::Range;

    use super::*;

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
    fn underscore_ok() {
        assert_ok("_x");
    }

    #[test]
    fn underscore_error() {
        assert_error("_", 0..1, IdentifierParseError::Underscore);
    }

    fn assert_ok(ident_str: &str) {
        assert_xid_chars(0, ident_str).unwrap();
    }

    fn assert_error(str: &str, expected_range: Range<usize>, expected_kind: IdentifierParseError) {
        let mock_offset = 10;

        let expected_error = ParseError::new(mock_offset, expected_range, expected_kind);

        let actual_error = assert_xid_chars(mock_offset, str).unwrap_err();

        assert_eq!(expected_error, actual_error, "input string: {str}");
    }
}
