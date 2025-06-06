use std::path::{Path, PathBuf};

use rusqlite_migration::Migrations;

use crate::*;

pub struct MainDb;

impl Db for MainDb {
    fn migrations() -> &'static Migrations<'static> {
        &MAIN_MIGRATIONS
    }
}

impl MainDb {
    pub(crate) fn path(dir: &Path) -> PathBuf {
        dir.join("main.sqlite")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path() {
        let expected = "/tmp/main.sqlite";

        let actual = MainDb::path(Path::new("/tmp"));
        assert_eq!(expected, actual.as_os_str());

        let actual = MainDb::path(Path::new("/tmp/"));
        assert_eq!(expected, actual.as_os_str());
    }
}
