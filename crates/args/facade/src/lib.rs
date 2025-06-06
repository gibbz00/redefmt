//! # `redefmt-args`

#![no_std]
// TEMP:
#![allow(missing_docs)]

pub use redefmt_args_internal::*;
// Almost exclusively used to declare tests within redefmt
#[doc(hidden)]
#[cfg(feature = "macros")]
pub use redefmt_args_macros::processed_format_string;
//
#[cfg(feature = "macros")]
pub use redefmt_args_macros::{deferred_format, format_string};
