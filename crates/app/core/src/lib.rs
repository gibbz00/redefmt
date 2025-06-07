//! redefmt-core

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod format;
pub use format::{Format, Formatter};

mod dispatcher;
pub use dispatcher::Dispatcher;
#[cfg(test)]
pub use dispatcher::NoopTestDispatcher;
#[cfg(feature = "testing")]
pub use dispatcher::{SharedTestDispatcher, SimpleTestDispatcher};

mod statement_writer;
pub use statement_writer::StatementWriter;

pub mod logger;
pub(crate) use logger::*;

pub mod codec;
pub(crate) use codec::*;

pub mod identifiers;
pub(crate) use identifiers::*;

#[cfg(feature = "db")]
mod sql_utils;
#[cfg(feature = "db")]
pub(crate) use sql_utils::sql_newtype;
