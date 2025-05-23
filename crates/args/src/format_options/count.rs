use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum FormatCountParseError {
    #[error("no closing '$' found")]
    UnclosedArgument,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatCount<'a> {
    Integer(Integer),
    #[cfg_attr(feature = "serde", serde(borrow))]
    Argument(FormatArgument<'a>),
}

impl FormatCount<'_> {
    pub(crate) fn owned(&self) -> FormatCount<'static> {
        match self {
            FormatCount::Integer(integer) => FormatCount::Integer(*integer),
            FormatCount::Argument(argument) => FormatCount::Argument(argument.owned()),
        }
    }
}
