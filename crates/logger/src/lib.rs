//! # `redefmt-logger`

#![no_std]
// TEMP:
#![allow(missing_docs)]

extern crate alloc;

mod write_value;
pub(crate) use write_value::WriteValue;

mod dispatcher;
pub(crate) use dispatcher::*;

mod stamper;
pub(crate) use stamper::Stamper;

mod global_registry;
pub use global_registry::{DispatcherHandle, GlobalRegistry};

mod global_logger;
pub(crate) use global_logger::GlobalLogger;
