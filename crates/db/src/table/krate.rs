use rusqlite::OptionalExtension;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrateId(ShortId);

sql_newtype!(CrateId);

#[derive(Debug, PartialEq)]
pub struct Crate<'a> {
    name: CrateName<'a>,
}

#[derive(Debug, thiserror::Error)]
#[error("encountered a crate record invariant error")]
pub enum CrateTableError {
    #[error(transparent)]
    Name(#[from] CrateNameError),
}

pub trait CrateTable {
    fn find_crate_record(&self, id: CrateId) -> Result<Option<Crate<'static>>, DbClientError>;

    fn insert_crate_record(&self, record: &Crate<'_>) -> Result<CrateId, DbClientError>;
}

impl CrateTable for DbClient<MainDb> {
    fn find_crate_record(&self, id: CrateId) -> Result<Option<Crate<'static>>, DbClientError> {
        let krate = self
            .connection
            .query_row("SELECT name FROM crate WHERE id = ?1", [id], |res| res.get(0))
            .optional()?
            .map(|name| Crate { name });

        Ok(krate)
    }

    fn insert_crate_record(&self, record: &Crate<'_>) -> Result<CrateId, DbClientError> {
        self.connection
            .query_row_and_then(
                "INSERT INTO crate(name) VALUES (?1) RETURNING id",
                [&record.name],
                |res| res.get(0),
            )
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_crate_id() -> CrateId {
        CrateId(ShortId(123))
    }

    fn mock_crate_record() -> Crate<'static> {
        Crate { name: CrateName::new("x").unwrap() }
    }

    #[test]
    fn find_and_create() {
        let (_dir_guard, client) = DbClient::mock_db();

        let mock_record = mock_crate_record();

        let id = client.insert_crate_record(&mock_record).unwrap();

        let found_record = client.find_crate_record(id).unwrap().unwrap();
        assert_eq!(mock_record, found_record);

        let fake_id = mock_crate_id();
        let found_record = client.find_crate_record(fake_id).unwrap();
        assert!(found_record.is_none());
    }

    #[test]
    fn unique_name_index() {
        let (_dir_guard, client) = DbClient::mock_db();

        let mock_record = mock_crate_record();

        let first_insert_result = client.insert_crate_record(&mock_record);
        assert!(first_insert_result.is_ok());

        let second_insert_result = client.insert_crate_record(&mock_record);
        assert!(second_insert_result.is_err());
    }
}
