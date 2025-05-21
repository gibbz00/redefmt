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

    pub fn owned(&self) -> FormatString<'static> {
        let segments = self.segments.iter().map(FormatStringSegment::owned).collect();
        FormatString { segments }
    }

    #[cfg(feature = "provided-args")]
    pub(crate) fn collect_args_mut(&mut self) -> Vec<&mut FormatArgument<'a>> {
        let mut format_string_args = Vec::new();
        let mut next_index = 0;

        for segment in &mut self.segments {
            if let FormatStringSegment::Format(format_segment) = segment {
                // disambiguate unnamed with indexed
                let arg = format_segment.argument.get_or_insert_with(|| {
                    let current_index = next_index;
                    next_index += 1;
                    FormatArgument::Index(Integer::new(current_index))
                });

                format_string_args.push(arg);

                if let Some(FormatCount::Argument(width_arg)) = &mut format_segment.options.width {
                    format_string_args.push(width_arg);
                }

                if let Some(FormatPrecision::Count(FormatCount::Argument(precision_arg))) =
                    &mut format_segment.options.precision
                {
                    format_string_args.push(precision_arg);
                }
            }
        }

        format_string_args
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
