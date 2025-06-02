mod core;
pub use core::FormatString;

mod parse_error;
pub use parse_error::{FormatStringParseError, FormatStringParseErrorKind};

pub mod segment;
pub(crate) use segment::{FormatArgumentSegment, FormatStringSegment, FormatStringSegmentError};

pub mod argument;
pub(crate) use argument::FormatArgument;

pub mod literal;
pub(crate) use literal::FormatLiteral;

pub mod options;
pub(crate) use options::*;

mod integer;
pub(crate) use integer::Integer;
