//! Sometimes referred to as the "wire-format", even if it can be written to any writer.

pub mod frame;
pub(crate) use frame::*;

pub mod encoding;
pub(crate) use encoding::*;
