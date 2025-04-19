//! # `redefmt-args`

#![no_std]

extern crate alloc;

mod parse;
pub(crate) use parse::{Parse, ParseError};

mod integer;
pub(crate) use integer::Integer;

mod identifier;
pub(crate) use identifier::{Identifier, IdentifierParseError};

mod argument;
pub(crate) use argument::Argument;

mod format_options;
pub(crate) use format_options::*;

mod format_argument;
pub(crate) use format_argument::FormatArgument;

mod format_string;
pub(crate) use format_string::{FormatString, FormatStringParseError};
