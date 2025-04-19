use crate::*;

/// Decimal integer that may contain leading zeroes, and must fit into an usize
pub struct Integer(usize);

impl Parse for Integer {
    fn parse(str: &str) -> Result<Self, ParseError> {
        usize::from_str_radix(str, 10)
            .map(Self)
            .map_err(|err| ParseError::new(0..str.len(), err))
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

    fn assert_parse(str: &str, expected: usize) {
        let integer = Integer::parse(str).unwrap();
        assert_eq!(expected, integer.0)
    }
}
