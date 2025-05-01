use crate::*;

#[derive(Debug, PartialEq)]
pub enum FormatPrecisionParseError {
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
