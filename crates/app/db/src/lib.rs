//! # `redefmt-db`

// TEMP:
#![allow(missing_docs)]

// foundational client modules

mod state_dir;
pub use state_dir::{StateDir, StateDirError};

mod migrations;
pub(crate) use migrations::{CRATE_MIGRATIONS, MAIN_MIGRATIONS};

mod db;
pub use db::{CrateDb, Db, MainDb};

mod client;
pub use client::{DbClient, DbClientError};

// table management modules

mod table;
pub use table::{Record, Table};

pub mod crate_table;
pub(crate) use crate_table::*;

pub mod statement_table;
pub(crate) use statement_table::*;

mod sql_utils;
pub(crate) use sql_utils::*;
