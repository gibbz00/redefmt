use std::path::Path;

use rusqlite::Connection;

use crate::*;

pub struct DbClient {
    connection: Connection,
}

#[derive(Debug, thiserror::Error)]
pub enum DbClientError {
    #[error("state directory resolution error")]
    StateDir(#[from] StateDirError),
    #[error("database open failure")]
    Open(#[source] rusqlite::Error),
}

impl DbClient {
    pub fn new(db: &DbType<'_>) -> Result<Self, DbClientError> {
        let dir = StateDir::resolve()?;
        Self::new_impl(&dir, db)
    }

    /// Separate to method to simplify tests by letting them pass a `TempDir` parameter
    pub(crate) fn new_impl(dir: &Path, db: &DbType<'_>) -> Result<Self, DbClientError> {
        let path = db.path(dir);
        let connection = Connection::open(path).map_err(DbClientError::Open)?;

        Ok(Self { connection })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = DbClient::new_impl(temp_dir.path(), &DbType::Main);

        assert!(result.is_ok());
    }
}
