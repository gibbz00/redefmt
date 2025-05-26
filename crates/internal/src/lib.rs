//! redefmt-internal

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

#[cfg(feature = "db")]
mod sql_utils;
#[cfg(feature = "db")]
pub(crate) use sql_utils::sql_newtype;
