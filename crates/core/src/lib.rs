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
#[doc(hidden)]
pub mod hidden_export {
    pub use redefmt_macros::{print, println, write, writeln};
}

#[cfg(feature = "macros")]
mod macros {
    #[macro_export]
    macro_rules! print {
        ($($arg:tt)*) => {
            $crate::hidden_export::print((file!(), module_path!(), line!()), $($arg:tt)*)
        };
    }

    #[macro_export]
    macro_rules! println {
        ($($arg:tt)*) => {
            $crate::hidden_export::print((file!(), module_path!(), line!()), $($arg:tt)*)
        };
    }

    #[macro_export]
    macro_rules! write {
        ($($arg:tt)*) => {
            $crate::hidden_export::write($($arg:tt)*)
        };
    }

    #[macro_export]
    macro_rules! writeln {
        ($($arg:tt)*) => {
            $crate::hidden_export::writeln($($arg:tt)*)
        };
    }
}
