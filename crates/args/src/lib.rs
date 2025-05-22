//! # `redefmt-args`

#![no_std]
// TEMP:
#![allow(missing_docs)]

extern crate alloc;

mod error;
pub use error::ParseError;

mod integer;
pub(crate) use integer::Integer;

pub mod identifier;
pub(crate) use identifier::{Identifier, IdentifierParseError};

mod format_argument;
pub(crate) use format_argument::FormatArgument;

mod format_options;
pub use format_options::FormatOptions;
pub(crate) use format_options::*;

mod format_segment;
pub(crate) use format_segment::{FormatSegment, FormatStringSegment};

mod format_string;
pub use format_string::FormatString;
pub(crate) use format_string::*;

#[cfg(feature = "provided-args")]
pub mod provided_args;
#[cfg(feature = "provided-args")]
pub(crate) use provided_args::*;

mod serde_utils;
