//! # `redefmt-args-internal`

#![no_std]
// TEMP:
#![allow(missing_docs)]

extern crate alloc;

mod error;
pub use error::FormatStringParseError;

mod format_string;
pub use format_string::FormatString;

pub mod format_segment;
pub(crate) use format_segment::{FormatSegment, FormatStringSegment, FormatStringSegmentError};

pub mod format_argument;
pub(crate) use format_argument::FormatArgument;

pub mod format_options;
pub(crate) use format_options::*;

pub mod identifier;
pub(crate) use identifier::*;

pub mod provided_args;
pub(crate) use provided_args::*;

mod integer;
pub(crate) use integer::Integer;

#[cfg(feature = "serde")]
mod serde_utils;

#[cfg(feature = "quote")]
mod quote_utils;
