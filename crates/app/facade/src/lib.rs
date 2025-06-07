//! redefmt - Redefined Deferred Formatting

#![cfg_attr(not(feature = "std"), no_std)]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "deferred")]
pub use deferred::*;
#[cfg(feature = "deferred")]
mod deferred {
    #[doc(hidden)]
    pub use redefmt_core::identifiers;
    pub use redefmt_core::{Format, Formatter, frame::Level, logger};
    pub use redefmt_macros::{Format, debug, error, info, log, print, println, trace, warn, write, writeln};
}

#[cfg(all(feature = "print-compat", not(feature = "deferred")))]
pub use std::{print, println};

#[cfg(all(feature = "log-compat", not(feature = "deferred")))]
pub use log::{Level, debug, error, info, log, trace, warn};
