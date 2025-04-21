use core::ops::Range;

use crate::*;

// Need not be sealed as trait should not be made public
pub trait Parse: Sized {
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError>;
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    // NOTE: character indices, not byte
    range: Range<usize>,
    kind: ParseErrorKind,
}

impl ParseError {
    pub(crate) fn new(offset: usize, range: Range<usize>, kind: impl Into<ParseErrorKind>) -> Self {
        let range = offset + range.start..offset + range.end;
        Self { range, kind: kind.into() }
    }

    pub(crate) fn new_char(offset: usize, kind: impl Into<ParseErrorKind>) -> Self {
        Self::new(offset, 0..1, kind)
    }
}

#[derive(Debug, PartialEq, derive_more::From)]
pub enum ParseErrorKind {
    Integer(core::num::ParseIntError),
    Identifier(IdentifierParseError),
    Count(FormatCountParseError),
    Precision(FormatPrecisionParseError),
    Trait(FormatTraitParseError),
    String(FormatStringParseError),
}
