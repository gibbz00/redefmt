//! redefmt - Redefined Deferred Formatting

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use redefmt_internal::{Format, Formatter};
#[cfg(feature = "derive")]
pub use redefmt_macros::Format;
