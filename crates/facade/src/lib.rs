//! redefmt - Redefined Deferred Formatting

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "deferred")]
mod deferred {
    #[cfg(feature = "deferred-macros")]
    pub use redefmt_macros::Format;
    //
    #[cfg(feature = "deferred-macros")]
    pub use redefmt_macros::{write, writeln};
    //
    mod export {
        #[doc(hidden)]
        pub use redefmt_core::identifiers;
        pub use redefmt_core::{Format, Formatter};
        #[cfg(feature = "deferred-logger")]
        pub use redefmt_core::{codec::frame::Level, logger};
    }
    pub use export::*;
    //
    #[cfg(feature = "deferred-logger")]
    pub use redefmt_macros::{debug, error, info, log, print, println, trace, warn};
}
#[cfg(feature = "deferred")]
pub use deferred::*;
