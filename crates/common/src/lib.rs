//! redefmt commons.

#![no_std]
// TEMP:
#![allow(missing_docs)]

pub const APPLICATION_NAME: &str = "redefmt";

pub mod codec;
pub(crate) use codec::*;
