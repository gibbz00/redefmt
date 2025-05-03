use std::{path::Path, sync::LazyLock};

use include_dir::include_dir;
use rusqlite::Connection;
use rusqlite_migration::Migrations;

use crate::*;

static MIGRATIONS_DIR: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

static MIGRATIONS: LazyLock<Migrations> =
    LazyLock::new(|| Migrations::from_directory(&MIGRATIONS_DIR).expect("invalid migration directory structure"));

pub struct DbClient {
    connection: Connection,
}

#[derive(Debug, thiserror::Error)]
pub enum DbClientError {
    #[error("state directory resolution error")]
    StateDir(#[from] StateDirError),
    #[error("internal database error")]
    Sqlite(#[from] rusqlite::Error),
    #[error("failed to apply migrations")]
    Migration(#[from] rusqlite_migration::Error),
}

impl DbClient {
    /// Create a new database client connection
    ///
    /// Creates the database if it does not exist, applies any outstanding
    /// migrations, and makes sure that that journal_mode=WAL with
    /// synchronous=NORMAL is used.
    pub fn new(db: &DbType<'_>) -> Result<Self, DbClientError> {
        let dir = StateDir::resolve()?;
        Self::new_impl(&dir, db)
    }

    /// Separate to method to simplify tests by letting them pass a `TempDir` parameter
    pub(crate) fn new_impl(dir: &Path, db: &DbType<'_>) -> Result<Self, DbClientError> {
        let path = db.path(dir);

        let mut connection = Connection::open(path)?;

        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;

        MIGRATIONS.to_latest(&mut connection)?;

        Ok(Self { connection })
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

        let temp_dir = tempfile::tempdir().unwrap();

        let client = DbClient::new_impl(temp_dir.path(), &DbType::Main).unwrap();

        assert_pragma(&client, "journal_mode", "wal".to_string());
        assert_pragma(&client, "synchronous", SYNCHRONOUS_NORMAL);

        fn assert_pragma<T: Debug + PartialEq + FromSql>(
            client: &DbClient,
            pragma_key: &str,
            expected_pragma_value: T,
        ) {
            client
                .connection
                .pragma_query_value(None, pragma_key, |res| {
                    let actual_pragma_value = res.get::<_, T>(0).unwrap();

                    assert_eq!(expected_pragma_value, actual_pragma_value);

                    Ok(())
                })
                .unwrap();
        }
    }
}
