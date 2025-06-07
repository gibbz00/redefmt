//! redefmt-core

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod identifiers;
pub(crate) use identifiers::*;

mod format;
pub use format::{Format, Formatter};

mod dispatcher;
pub use dispatcher::*;

pub mod write;
pub(crate) use write::*;

pub mod logger;
pub(crate) use logger::*;

pub mod frame;
pub(crate) use frame::*;
