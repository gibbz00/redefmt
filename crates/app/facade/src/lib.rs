//! redefmt - Redefined Deferred Formatting

#![cfg_attr(not(feature = "std"), no_std)]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "deferred")]
mod deferred {
    pub use redefmt_macros::Format;
    //
    mod export {
        #[doc(hidden)]
        pub use redefmt_core::identifiers;
        #[cfg(feature = "deferred-logger")]
        pub use redefmt_core::{codec::frame::Level, logger};
    }
    pub use export::*;
    //
    #[cfg(feature = "deferred-logger-macros")]
    pub use redefmt_macros::{debug, error, info, log, print, println, trace, warn};
}
#[cfg(feature = "deferred")]
pub use deferred::*;

mod base {
    pub use redefmt_core::{Dispatcher, Format, Formatter};
}
pub use base::*;
#[cfg(feature = "deferred")]
pub use redefmt_macros::{write, writeln};

#[cfg(not(feature = "deferred"))]
#[macro_export]
macro_rules! write {
    ($($tt:tt)*) => {{
        use ::core::fmt::Write as  _;
        ::core::write!($($tt)*)
    }};
}

#[cfg(not(feature = "deferred"))]
#[macro_export]
macro_rules! writeln {
    ($($tt:tt)*) => {{
        use ::core::fmt::Write as  _;
        ::core::writeln!($($tt)*)
    }};
}

#[cfg(all(feature = "print-compat", not(feature = "deferred")))]
pub use std::{print, println};

#[cfg(all(feature = "log-compat", not(feature = "deferred")))]
pub use log::{Level, debug, error, info, log, trace, warn};
