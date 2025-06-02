use alloc::string::String;

use crate::*;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum FormatStringProcessorError {
    #[error("duplicate argument names not allowed, namely: {0}")]
    ProvidedDuplicate(ArgumentIdentifier<'static>),
    #[error("invalid positional argument {0}, provided {1}, positional argument are zero-based")]
    InvalidStringPositional(usize, usize),
    #[error("provided {0} unused positional arguments")]
    UnusedPositionals(usize),
    #[error("provided named argument {0} not used in format string")]
    UnusedNamed(String),
    #[error("missing named argument {0} in provided arguments")]
    MissingNamed(String),
}
