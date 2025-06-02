//! # `redefmt-printer`

// TEMP:
#![allow(missing_docs)]

pub mod configuration;
pub(crate) use configuration::*;

mod printer;
pub use printer::PrettyPrinter;
