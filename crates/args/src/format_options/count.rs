use crate::*;

#[derive(Debug, PartialEq)]
pub enum FormatCountParseError {
    UnclosedArgument,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormatCount<'a> {
    Integer(Integer),
    Argument(Argument<'a>),
}

impl FormatCount<'_> {
    pub(crate) fn owned(&self) -> FormatCount<'static> {
        match self {
            FormatCount::Integer(integer) => FormatCount::Integer(*integer),
            FormatCount::Argument(argument) => FormatCount::Argument(argument.owned()),
        }
    }
}
