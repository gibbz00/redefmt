//! redefmt-core

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "logger")]
pub mod logger;
#[cfg(feature = "logger")]
pub(crate) use logger::*;

pub mod codec;
pub(crate) use codec::*;

mod format;
pub use format::{Format, Formatter};

pub mod identifiers;
pub(crate) use identifiers::*;

pub mod level {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum Level {
        Error,
        Warn,
        Info,
        Debug,
        Trace,
    }
}

#[cfg(feature = "db")]
mod sql_utils;
#[cfg(feature = "db")]
pub(crate) use sql_utils::sql_newtype;
