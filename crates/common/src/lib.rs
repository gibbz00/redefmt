//! redefmt commons.

#![no_std]
// TEMP:
#![allow(missing_docs)]

pub const APPLICATION_NAME: &str = "redefmt";

pub mod codec;
pub(crate) use codec::*;

pub mod identifiers;
pub(crate) use identifiers::*;

#[cfg(feature = "db")]
mod sql_utils;
#[cfg(feature = "db")]
pub(crate) use sql_utils::sql_newtype;
