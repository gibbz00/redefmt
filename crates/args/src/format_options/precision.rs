use crate::*;

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("failed to parse precision format option")]
pub enum FormatPrecisionParseError {
    #[error("empty string after '.', expected a precision count or argument")]
    Empty,
}

/// https://doc.rust-lang.org/std/fmt/index.html#precision
#[derive(Debug, PartialEq)]
pub enum FormatPrecision<'a> {
    Count(FormatCount<'a>),
    /// '*'
    NextArgument,
}

impl FormatPrecision<'_> {
    pub(crate) fn owned(&self) -> FormatPrecision<'static> {
        match self {
            FormatPrecision::Count(count) => FormatPrecision::Count(count.owned()),
            FormatPrecision::NextArgument => FormatPrecision::NextArgument,
        }
    }
}
