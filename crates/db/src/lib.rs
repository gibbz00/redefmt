//! # `redefmt-db`

// TEMP:
#![allow(missing_docs)]

mod state_dir;
pub(crate) use state_dir::{StateDir, StateDirError};

mod migrations;
pub(crate) use migrations::{CRATE_MIGRATIONS, MAIN_MIGRATIONS};

mod db;
pub(crate) use db::*;

mod client;
pub use client::{DbClient, DbClientError};

mod columns;
pub(crate) use columns::*;

mod table;
pub(crate) use table::*;

mod sql_utils;
pub(crate) use sql_utils::*;

// TEMP: for sketching print statement, move to column and table modules
mod print_info;
pub(crate) use print_info::PrintInfo;
mod documents;
pub(crate) use documents::*;
