use crate::*;

/// Decimal integer that may contain leading zeroes, and must fit into an usize
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Integer(pub(crate) usize);

impl Parse for Integer {
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        str.parse::<usize>()
            .map(Self)
            .map_err(|err| ParseError::new(offset, 0..str.len(), err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        // decimal radix
        assert_parse("10", 10);
        // with leading zeros
        assert_parse("0123", 123);
    }

    #[test]
    fn parse_error() {
        let invalid = "FF";
        let mock_offset = 10;

        let expected_error = {
            let parse_int_error = invalid.parse::<usize>().unwrap_err();
            ParseError::new(mock_offset, 0..invalid.len(), parse_int_error)
        };

        let actual_error = Integer::parse(mock_offset, invalid).unwrap_err();

        assert_eq!(expected_error, actual_error);
    }

    fn assert_parse(str: &str, expected: usize) {
        let integer = Integer::parse(0, str).unwrap();
        assert_eq!(expected, integer.0)
    }
}
