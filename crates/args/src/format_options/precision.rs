use crate::*;

/// https://doc.rust-lang.org/std/fmt/index.html#precision
#[derive(Debug, PartialEq)]
pub enum FormatPrecision {
    Count(FormatCount),
    /// '*'
    NextArgument,
}

#[derive(Debug, PartialEq)]
pub enum FormatPrecisionParseError {
    Empty,
}

impl Parse for FormatPrecision {
    /// Context from `FormatOptions::parse`
    /// - No leading '.'
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        if str.is_empty() {
            // `- 1` to point to '.' character
            return Err(ParseError::new_char(offset - 1, FormatPrecisionParseError::Empty));
        }

        if str == "*" {
            return Ok(Self::NextArgument);
        }

        FormatCount::parse(offset, str).map(Self::Count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_next() {
        let actual = FormatPrecision::parse(0, "*").unwrap();

        let expected = FormatPrecision::NextArgument;

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_count() {
        let actual = FormatPrecision::parse(0, "01$").unwrap();

        let expected = {
            let count_argument = FormatCount::parse(0, "01$").unwrap();
            FormatPrecision::Count(count_argument)
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn empty_error() {
        let actual = FormatPrecision::parse(1, "").unwrap_err();
        let expected = ParseError::new_char(0, FormatPrecisionParseError::Empty);

        assert_eq!(actual, expected)
    }
}
