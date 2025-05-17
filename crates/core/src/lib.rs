//! redefmt - Redefined Deferred Formatting

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "derive")]
pub use redefmt_macros::Format;

pub mod codec;
pub(crate) use codec::*;

pub mod identifiers;
pub(crate) use identifiers::*;

mod format;
pub use format::{Format, Formatter};

#[cfg(feature = "db")]
mod sql_utils;
#[cfg(feature = "db")]
pub(crate) use sql_utils::sql_newtype;
