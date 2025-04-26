//! # `redefmt-logger`

#![no_std]
// TEMP:
#![allow(missing_docs)]

mod write_primitive;
pub(crate) use write_primitive::WritePrimitive;

mod dispatcher;
pub(crate) use dispatcher::Dispatcher;

mod stamper;
pub(crate) use stamper::Stamper;

mod global_registry;
pub use global_registry::GlobalRegistry;

mod global_logger;
pub(crate) use global_logger::GlobalLogger;
