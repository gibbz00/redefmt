//! # `redefmt-args`

#![no_std]
// TEMP:
#![allow(missing_docs)]

pub use redefmt_args_internal::*;
#[cfg(feature = "macros")]
pub use redefmt_args_macros::*;
