//! # `redefmt-logger`

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

// re-export of core library items used within the macros
#[doc(hidden)]
pub use redefmt_internal::{
    Format,
    identifiers::{CrateId, PrintStatementId},
};
//
pub use redefmt_macros::{print, println};

mod global_logger;
pub use global_logger::{GlobalLogger, GlobalLoggerError};

mod global_stamper;
pub(crate) use global_stamper::GlobalStamper;

mod global_dispatcher;
pub(crate) use global_dispatcher::{GlobalDispatcher, GlobalDispatcherHandle};

mod stamper;
pub(crate) use stamper::Stamper;
#[cfg(test)]
pub(crate) use stamper::TestStamper;
