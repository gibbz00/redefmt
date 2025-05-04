use crate::*;

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("failed to parse count argument")]
pub enum FormatCountParseError {
    #[error("no closing '$' found")]
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
