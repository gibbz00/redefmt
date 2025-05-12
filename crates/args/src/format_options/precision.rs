use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
#[error("failed to parse precision format option")]
pub enum FormatPrecisionParseError {
    #[error("empty string after '.', expected a precision count or argument")]
    Empty,
}

/// <https://doc.rust-lang.org/std/fmt/index.html#precision>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatPrecision<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
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
