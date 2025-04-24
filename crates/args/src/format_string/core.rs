use alloc::{borrow::Cow, vec::Vec};

use crate::*;

#[derive(Debug, PartialEq)]
pub struct FormatString<'a> {
    pub(crate) segments: Vec<FormatStringSegment<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum FormatStringSegment<'a> {
    Literal(Cow<'a, str>),
    Format(FormatSegment<'a>),
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
