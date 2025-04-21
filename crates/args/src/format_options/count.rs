use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum FormatCount {
    Integer(Integer),
    Argument(Argument),
}

#[derive(Debug, PartialEq)]
pub enum FormatCountParseError {
    UnclosedArgument,
}
