//! redefmt - Redefined Deferred Formatting

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "macros")]
pub use redefmt_macros::Format;
//
#[cfg(feature = "macros")]
pub use redefmt_macros::{write, writeln};
//
mod export {
    pub use redefmt_core::{Format, Formatter, identifiers};
    #[cfg(feature = "logger")]
    pub use redefmt_core::{codec::frame::Level, logger};
}
pub use export::*;
//
#[cfg(feature = "logger")]
pub use redefmt_macros::{debug, error, info, log, print, println, trace, warn};
