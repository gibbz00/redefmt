//! # `redefmt-args`

#![no_std]
// TEMP:
#![allow(missing_docs)]

extern crate alloc;

mod error;
pub use error::ParseError;

mod integer;
pub(crate) use integer::Integer;

mod identifier;
pub(crate) use identifier::{Identifier, IdentifierParseError};

mod argument;
pub(crate) use argument::Argument;

mod format_options;
pub use format_options::FormatOptions;
pub(crate) use format_options::*;

mod format_segment;
pub(crate) use format_segment::FormatSegment;

mod format_string;
pub use format_string::FormatString;
pub(crate) use format_string::*;

mod argument_register;
pub use argument_register::RequiredArgumentsError;
pub(crate) use argument_register::{ArgumentRegister, RequiredArguments};
