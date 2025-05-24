//! redefmt - Redefined Deferred Formatting

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "macros")]
pub use redefmt_macros::Format;

mod export {
    pub use redefmt_internal::{Format, Formatter, identifiers};
}
pub use export::*;
#[cfg(feature = "macros")]
pub use redefmt_macros::{write, writeln};
