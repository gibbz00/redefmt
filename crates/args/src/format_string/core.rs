use alloc::vec::Vec;

use crate::*;

#[derive(Debug, PartialEq)]
pub struct FormatString<'a> {
    pub(crate) segments: Vec<FormatStringSegment<'a>>,
}

impl<'a> FormatString<'a> {
    pub fn parse(str: &'a str) -> Result<Self, ParseError> {
        Self::parse_impl(0, str)
    }

    /// Verify that format string parameters are consistent, returning their requirements
    pub fn required_parameters(&self) -> Result<RequiredArguments, RequiredArgumentsError> {
        ArgumentRegister::new(self).resolve()
    }
}

#[derive(Debug, PartialEq)]
pub enum FormatStringSegment<'a> {
    Literal(&'a str),
    Format(FormatSegment<'a>),
}
