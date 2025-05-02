//! # `redefmt-db`

// TEMP:
#![allow(missing_docs)]

extern crate alloc;

mod client;
pub use client::DbClient;

mod write_content;
pub(crate) use write_content::*;

mod print_info;
pub(crate) use print_info::PrintInfo;

mod documents;
pub(crate) use documents::*;
