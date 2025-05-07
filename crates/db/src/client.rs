use std::{marker::PhantomData, path::Path};

use rusqlite::Connection;

use crate::*;

pub struct DbClient<D: Db> {
    pub(crate) connection: Connection,
    marker: PhantomData<D>,
}

#[derive(Debug, thiserror::Error)]
pub enum DbClientError {
    #[error("state directory resolution error")]
    StateDir(#[from] StateDirError),
    #[error("internal database error")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json serialization error")]
    Json(#[from] serde_json::Error),
    #[error("failed to apply migrations")]
    Migration(#[from] rusqlite_migration::Error),
    // IMPROVEMENT: only possible from MainDb
    #[error(transparent)]
    CrateTable(#[from] CrateTableError),
}

impl DbClient<MainDb> {
    pub fn new_main() -> Result<Self, DbClientError> {
        let dir = StateDir::resolve()?;
        Self::new_main_impl(&dir)
    }

    /// Separate to method to simplify tests by letting them pass a `TempDir` parameter
    pub(crate) fn new_main_impl(dir: &Path) -> Result<Self, DbClientError> {
        let path = MainDb::path(dir);
        Self::init(&path)
    }
}

impl DbClient<CrateDb> {
    pub fn new_crate(crate_name: &CrateName) -> Result<Self, DbClientError> {
        let dir = StateDir::resolve()?;
        Self::new_crate_impl(&dir, crate_name)
    }

    /// Separate to method to simplify tests by letting them pass a `TempDir` parameter
    pub(crate) fn new_crate_impl(dir: &Path, crate_name: &CrateName) -> Result<Self, DbClientError> {
        let path = CrateDb::path(dir, crate_name);
        Self::init(&path)
    }
}

impl<D: Db> DbClient<D> {
    /// Create a new database client connection
    ///
    /// Creates the database if it does not exist, applies any outstanding
    /// migrations, and makes sure that that journal_mode=WAL with
    /// synchronous=NORMAL is used.
    fn init(path: &Path) -> Result<Self, DbClientError> {
        let mut connection = Connection::open(path)?;

        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;

        D::migrations().to_latest(&mut connection)?;

        Ok(Self { connection, marker: PhantomData })
    }
}

#[cfg(test)]
mod mock {
    use tempfile::TempDir;

    use super::*;

    impl<D: Db> DbClient<D> {
        pub fn mock_db() -> (TempDir, Self) {
            let temp_dir = tempfile::tempdir().unwrap();
            let temp_path = temp_dir.path().join("test.sqlite");

            let db = Self::init(&temp_path).unwrap();

            (temp_dir, db)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use rusqlite::types::FromSql;

    use super::*;

    #[test]
    fn new_with_pragma() {
        const SYNCHRONOUS_NORMAL: usize = 1;

        let (_dir_guard, db) = DbClient::<MainDb>::mock_db();

        assert_pragma(&db, "journal_mode", "wal".to_string());
        assert_pragma(&db, "synchronous", SYNCHRONOUS_NORMAL);

        fn assert_pragma<T: Debug + PartialEq + FromSql>(
            db: &DbClient<impl Db>,
            pragma_key: &str,
            expected_pragma_value: T,
        ) {
            db.connection
                .pragma_query_value(None, pragma_key, |res| {
                    let actual_pragma_value = res.get::<_, T>(0).unwrap();

                    assert_eq!(expected_pragma_value, actual_pragma_value);

                    Ok(())
                })
                .unwrap();
        }
    }
}
