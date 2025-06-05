//! redefmt-core

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "deferred-alloc")]
extern crate alloc;

#[cfg(feature = "deferred")]
mod deferred;
#[cfg(feature = "deferred")]
pub use deferred::*;
