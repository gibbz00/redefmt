use alloc::{
    borrow::{Cow, ToOwned},
    string::ToString,
    vec::Vec,
};

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

    pub fn owned(&self) -> FormatString<'static> {
        let segments = self.segments.iter().map(FormatStringSegment::owned).collect();
        FormatString { segments }
    }
}

#[derive(Debug, PartialEq)]
pub enum FormatStringSegment<'a> {
    Literal(Cow<'a, str>),
    Format(FormatSegment<'a>),
}

impl FormatStringSegment<'_> {
    pub(crate) fn owned(&self) -> FormatStringSegment<'static> {
        match self {
            FormatStringSegment::Literal(cow) => FormatStringSegment::Literal(Cow::Owned(cow.to_string())),
            FormatStringSegment::Format(segment) => FormatStringSegment::Format(segment.owned()),
        }
    }
}
