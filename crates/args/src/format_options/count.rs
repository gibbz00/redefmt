use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum FormatCount<'a> {
    Integer(Integer),
    Argument(Argument<'a>),
}

#[derive(Debug, PartialEq)]
pub enum FormatCountParseError {
    UnclosedArgument,
}
