//! # `redefmt-args-internal`

#![no_std]
// TEMP:
#![allow(missing_docs)]

extern crate alloc;

mod error;
pub use error::ParseError;

mod integer;
pub(crate) use integer::Integer;

mod identifier;
pub(crate) use identifier::*;

mod format_argument;
pub(crate) use format_argument::FormatArgument;

mod format_options;
pub(crate) use format_options::*;

mod format_segment;
pub(crate) use format_segment::{FormatSegment, FormatStringSegment};

mod format_string;
pub use format_string::{FormatString, FormatStringParseError};

#[cfg(feature = "provided-args")]
pub mod provided_args;
#[cfg(feature = "provided-args")]
pub(crate) use provided_args::*;

mod serde_utils;
