use std::sync::LazyLock;

use include_dir::include_dir;
use rusqlite_migration::Migrations;

static MAIN_DIR: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations/main");

pub static MAIN_MIGRATIONS: LazyLock<Migrations> =
    LazyLock::new(|| Migrations::from_directory(&MAIN_DIR).expect(ERROR_MESSAGE));

static CRATE_DIR: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations/crate");

pub static CRATE_MIGRATIONS: LazyLock<Migrations> =
    LazyLock::new(|| Migrations::from_directory(&CRATE_DIR).expect(ERROR_MESSAGE));

const ERROR_MESSAGE: &str = "invalid migration directory structure";
