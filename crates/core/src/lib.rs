//! redefmt - Redefined Deferred Formatting

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "derive")]
pub use redefmt_macros::Format;

mod export {
    pub use redefmt_internal::{Format, Formatter, identifiers};
}
pub use export::*;
