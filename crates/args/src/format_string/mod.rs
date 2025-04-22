mod core;
pub use core::FormatString;
pub(crate) use core::FormatStringSegment;

mod parse;
pub use parse::FormatStringParseError;
