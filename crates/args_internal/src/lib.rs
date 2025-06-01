//! # `redefmt-args-internal`

#![no_std]
// TEMP:
#![allow(missing_docs)]

extern crate alloc;

pub mod identifier;
pub(crate) use identifier::*;

pub mod format_string;
pub(crate) use format_string::*;

mod provided_args;
pub(crate) use provided_args::ProvidedArgs;

pub mod resolver;
pub(crate) use resolver::*;

mod format_expression;
pub use format_expression::FormatExpression;

pub mod deferred;
pub(crate) use deferred::*;

#[cfg(feature = "serde")]
mod serde_utils;

#[cfg(feature = "quote")]
mod quote_utils;
