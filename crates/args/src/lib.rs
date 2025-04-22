//! # `redefmt-args`

#![no_std]
// TEMP:
#![allow(missing_docs)]

extern crate alloc;

mod error;
pub(crate) use error::ParseError;

mod integer;
pub(crate) use integer::Integer;

mod identifier;
pub(crate) use identifier::{Identifier, IdentifierParseError};

mod argument;
pub(crate) use argument::Argument;

mod format_options;
pub(crate) use format_options::*;

mod format_segment;
pub(crate) use format_segment::FormatSegment;

mod format_string;
pub(crate) use format_string::*;
pub use format_string::{FormatString, FormatStringParseError};

mod argument_register;
pub(crate) use argument_register::{ArgumentRegister, RequiredArguments, RequiredArgumentsError};
