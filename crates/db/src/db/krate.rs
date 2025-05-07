use std::path::{Path, PathBuf};

use rusqlite_migration::Migrations;

use crate::*;

pub struct CrateDb;

impl Db for CrateDb {
    fn migrations() -> &'static Migrations<'static> {
        &CRATE_MIGRATIONS
    }
}

impl CrateDb {
    pub(crate) fn path(dir: &Path, crate_name: &CrateName) -> Result<PathBuf, DbClientError> {
        let crate_dir = dir.join("crates").join(crate_name.as_ref());

        if !crate_dir.exists() {
            std::fs::create_dir_all(&crate_dir).map_err(|err| DbClientError::CrateDir(crate_dir.clone(), err))?;
        }

        Ok(crate_dir.join("db.sqlite"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path() {
        let expected = "/tmp/crates/abc/db.sqlite";

        let dir = Path::new("/tmp");
        let crate_name = CrateName::new("abc").unwrap();
        let actual = CrateDb::path(&dir, &crate_name).unwrap();

        assert_eq!(expected, actual.as_os_str());
    }
}
