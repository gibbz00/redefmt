//! redefmt - Redefined Deferred Formatting

// TEMP:
#![allow(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "deferred")]
pub use deferred::*;
#[cfg(feature = "deferred")]
mod deferred {
    #[doc(hidden)]
    pub use redefmt_core::identifiers;
    pub use redefmt_core::{Format, Formatter, frame::Level, logger};
    pub use redefmt_macros::{Format, write, writeln};
}

#[allow(unused_imports)]
pub use printing::*;
#[rustfmt::skip]
mod printing {
    #[cfg(all(feature = "print", not(feature = "deferred")))]
    pub use std::{print, println};

    #[cfg(all(not(feature = "print"), feature = "deferred"))]
    pub use redefmt_macros::{print, println};

    #[cfg(all(feature = "print", feature = "deferred"))]
    pub use redefmt_macros::{print_compat as print, println_compat as println};

    #[cfg(all(feature = "print", feature = "deferred"))]
    #[doc(hidden)]
    pub use std::print as redefmt_to_print;

    #[cfg(all(feature = "print", feature = "deferred"))]
    #[doc(hidden)]
    pub use std::println as redefmt_to_println;
}

#[allow(unused_imports)]
pub use logging::*;
#[rustfmt::skip]
mod logging {
    #[cfg(all(feature = "log", not(feature = "deferred")))]
    pub use log::{Level, debug, error, info, log, trace, warn};

    #[cfg(all(not(feature = "log"), feature = "deferred"))]
    pub use redefmt_macros::{debug, error, info, log, trace, warn};

    #[cfg(all(feature = "log", feature = "deferred"))]
    pub use redefmt_macros::{
        debug_compat as debug,
        error_compat as error,
        info_compat as info,
        log_compat as log,
        trace_compat as trace,
        warn_compat as warn,
    };

    #[cfg(all(feature = "log", feature = "deferred"))]
    #[doc(hidden)]
    pub use log::log as redefmt_to_log;

    // Don't want to pull in `log` into `redefmt_core`, hence the newtype
    #[cfg(all(feature = "log", feature = "deferred"))]
    #[doc(hidden)]
    pub struct LevelCompat(pub redefmt_core::frame::Level);

    #[cfg(all(feature = "log", feature = "deferred"))]
    #[doc(hidden)]
    impl From<LevelCompat> for log::Level {
        fn from(value: LevelCompat) -> Self {
            use redefmt_core::frame::Level;

            match value.0 {
                Level::Trace => log::Level::Trace ,
                Level::Debug => log::Level::Debug ,
                Level::Info => log::Level::Info ,
                Level::Warn => log::Level::Warn ,
                Level::Error => log::Level::Error ,
            }
        }
    }
}
