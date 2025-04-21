use crate::*;

#[derive(Debug, PartialEq)]
pub enum FormatCount {
    Integer(Integer),
    Argument(Argument),
}

impl Parse for FormatCount {
    /// Context from `FormatPrecision::parse`
    /// - `str` not empty
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        match str.strip_suffix('$') {
            Some(argument_str) => Argument::parse(offset, argument_str).map(FormatCount::Argument),
            None => Integer::parse(offset, str).map(FormatCount::Integer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_argument() {
        let actual = FormatCount::parse(0, "x$").unwrap();

        let expected = {
            let argument = Argument::parse(0, "x").unwrap();
            FormatCount::Argument(argument)
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_count() {
        let actual = FormatCount::parse(0, "01").unwrap();

        let expected = {
            let integer = Integer::parse(0, "01").unwrap();
            FormatCount::Integer(integer)
        };

        assert_eq!(expected, actual);
    }
}
