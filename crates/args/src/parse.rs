use core::ops::Range;

use crate::*;

pub trait Parse: private::Sealed + Sized {
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError>;
}

mod private {
    use crate::*;

    pub trait Sealed {}

    impl Sealed for Integer {}
    impl Sealed for Identifier {}
    impl Sealed for Argument {}
    impl Sealed for FormatTrait {}
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    range: Range<usize>,
    kind: ParseErrorKind,
}

impl ParseError {
    pub(crate) fn new(offset: usize, range: Range<usize>, kind: impl Into<ParseErrorKind>) -> Self {
        let range = offset + range.start..offset + range.end;
        Self { range, kind: kind.into() }
    }
}

#[derive(Debug, PartialEq, derive_more::From)]
pub enum ParseErrorKind {
    Integer(core::num::ParseIntError),
    Identifier(IdentifierParseError),
    FormatTrait(FormatTraitParseError),
}
