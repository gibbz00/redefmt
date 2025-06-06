//! redefmt-core

#![no_std]
// TEMP:
#![allow(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "deferred")]
mod deferred;
#[cfg(feature = "deferred")]
pub use deferred::*;

// base
mod base;
pub use base::*;
