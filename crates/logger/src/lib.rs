//! # `redefmt-logger`

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod write_value;
pub(crate) use write_value::WriteValue;

mod stamper;
pub(crate) use stamper::Stamper;
#[cfg(test)]
pub(crate) use stamper::TestStamper;

mod global_stamper;
pub(crate) use global_stamper::GlobalStamper;

mod global_dispatcher;
pub use global_dispatcher::{GlobalDispatcher, GlobalDispatcherHandle};

mod global_logger;
pub use global_logger::GlobalLogger;
