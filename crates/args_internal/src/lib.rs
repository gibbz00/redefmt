//! # `redefmt-args-internal`

#![no_std]
// TEMP:
#![allow(missing_docs)]

extern crate alloc;

mod parse_error;
pub use parse_error::FormatStringParseError;

mod format_string;
pub use format_string::FormatString;

pub mod format_segment;
pub(crate) use format_segment::{FormatArgumentSegment, FormatStringSegment, FormatStringSegmentError};

pub mod format_argument;
pub(crate) use format_argument::FormatArgument;

pub mod format_literal;
pub(crate) use format_literal::FormatLiteral;

pub mod format_options;
pub(crate) use format_options::*;

pub mod identifier;
pub(crate) use identifier::*;

mod resolver;
pub(crate) use resolver::*;

#[cfg(feature = "syn")]
pub mod provided_args;
#[cfg(feature = "syn")]
pub(crate) use provided_args::*;

#[cfg(feature = "syn")]
mod format_expression;
#[cfg(feature = "syn")]
pub use format_expression::FormatExpression;

pub mod deferred;
pub(crate) use deferred::*;

mod integer;
pub(crate) use integer::Integer;

#[cfg(feature = "serde")]
mod serde_utils;

#[cfg(feature = "quote")]
mod quote_utils;
