use alloc::{borrow::Cow, string::ToString, vec::Vec};

use crate::*;

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct FormatString<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
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

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatStringSegment<'a> {
    Literal(Cow<'a, str>),
    #[cfg_attr(feature = "serde", serde(borrow))]
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
