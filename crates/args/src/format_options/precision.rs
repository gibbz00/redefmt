use crate::*;

/// https://doc.rust-lang.org/std/fmt/index.html#precision
#[derive(Debug, PartialEq)]
pub enum FormatPrecision<'a> {
    Count(FormatCount<'a>),
    /// '*'
    NextArgument,
}

#[derive(Debug, PartialEq)]
pub enum FormatPrecisionParseError {
    Empty,
}
