use core::ops::Range;

use crate::*;

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

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParseErrorKind {
    #[error("failed to parse integer")]
    Integer(#[from] core::num::ParseIntError),
    #[error(transparent)]
    Identifier(#[from] IdentifierParseError),
    #[error(transparent)]
    Count(#[from] FormatCountParseError),
    #[error(transparent)]
    Precision(#[from] FormatPrecisionParseError),
    #[error(transparent)]
    Trait(#[from] FormatTraitParseError),
    #[error(transparent)]
    String(#[from] FormatStringParseError),
}
