use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatArgument<'a> {
    Index(Integer),
    #[cfg_attr(feature = "serde", serde(borrow))]
    Identifier(ArgumentIdentifier<'a>),
}

impl<'a> FormatArgument<'a> {
    /// Context from `FormatSegment::parse`:
    /// - `str` not empty
    pub(crate) fn parse(offset: usize, str: &'a str) -> Result<Self, ParseError> {
        match str.starts_with(|ch: char| ch.is_ascii_digit()) {
            true => Integer::parse(offset, str).map(FormatArgument::Index),
            false => ArgumentIdentifier::parse_impl(offset, str).map(FormatArgument::Identifier),
        }
    }

    pub(crate) fn owned(&self) -> FormatArgument<'static> {
        match self {
            FormatArgument::Index(integer) => FormatArgument::Index(*integer),
            FormatArgument::Identifier(identifier) => FormatArgument::Identifier(identifier.owned()),
        }
    }

    pub(crate) fn matches_name(&self, identifier: &ArgumentIdentifier) -> bool {
        match self {
            FormatArgument::Index(_) => false,
            FormatArgument::Identifier(argument_identifier) => argument_identifier == identifier,
        }
    }

    pub(crate) fn matches_index(&self, index: usize) -> bool {
        match self {
            FormatArgument::Index(argument_index) => argument_index.inner() == index,
            FormatArgument::Identifier(_) => false,
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
            FormatArgument::Index(integer)
        };

        let actual_argument = FormatArgument::parse(0, str).unwrap();

        assert_eq!(expected_argument, actual_argument);
    }

    #[test]
    fn parse_identifier() {
        let str = "x";

        let expected_argument = {
            let identifier = ArgumentIdentifier::parse(str).unwrap();
            FormatArgument::Identifier(identifier)
        };

        let actual_argument = FormatArgument::parse(0, str).unwrap();

        assert_eq!(expected_argument, actual_argument);
    }
}
