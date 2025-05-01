use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Argument<'a> {
    Index(Integer),
    Identifier(Identifier<'a>),
}

impl<'a> Argument<'a> {
    /// Context from `FormatSegment::parse`:
    /// - `str` not empty
    pub(crate) fn parse(offset: usize, str: &'a str) -> Result<Self, ParseError> {
        match str.starts_with(|ch: char| ch.is_ascii_digit()) {
            true => Integer::parse(offset, str).map(Argument::Index),
            false => Identifier::parse(offset, str).map(Argument::Identifier),
        }
    }

    pub(crate) fn owned(&self) -> Argument<'static> {
        match self {
            Argument::Index(integer) => Argument::Index(*integer),
            Argument::Identifier(identifier) => Argument::Identifier(identifier.owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_index() {
        let str = "01";

        let expected_argument = {
            let integer = Integer::parse(0, str).unwrap();
            Argument::Index(integer)
        };

        let actual_argument = Argument::parse(0, str).unwrap();

        assert_eq!(expected_argument, actual_argument);
    }

    #[test]
    fn parse_identifier() {
        let str = "x";

        let expected_argument = {
            let identifier = Identifier::parse(0, str).unwrap();
            Argument::Identifier(identifier)
        };

        let actual_argument = Argument::parse(0, str).unwrap();

        assert_eq!(expected_argument, actual_argument);
    }
}
