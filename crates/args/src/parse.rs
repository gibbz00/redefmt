use core::ops::Range;

use crate::*;

// TODO: remove sealed requirement given that trait should only be pub(crate)?
pub trait Parse: private::Sealed + Sized {
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError>;
}

mod private {
    use crate::*;

    pub trait Sealed {}

    macro_rules! impl_sealed {
        ($($ident:ty,)*) => {
            $(impl Sealed for $ident {})*
        };
    }

    impl_sealed!(Integer, Identifier, Argument, FormatTrait, FormatArgument, FormatString,);
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
    FormatString(FormatStringParseError),
}
