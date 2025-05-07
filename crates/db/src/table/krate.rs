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
    fn find_crate_by_id(&self, id: CrateId) -> Result<Option<Crate<'static>>, DbClientError>;

    fn find_crate_by_name(&self, name: &CrateName<'_>) -> Result<Option<(CrateId, Crate<'static>)>, DbClientError>;

    fn insert_crate_record(&self, record: &Crate<'_>) -> Result<CrateId, DbClientError>;
}

impl CrateTable for DbClient<MainDb> {
    fn find_crate_by_id(&self, id: CrateId) -> Result<Option<Crate<'static>>, DbClientError> {
        let krate = self
            .connection
            .query_row("SELECT name FROM crate WHERE id = ?1", [id], |res| res.get(0))
            .optional()?
            .map(|name| Crate { name });

        Ok(krate)
    }

    fn find_crate_by_name(&self, name: &CrateName<'_>) -> Result<Option<(CrateId, Crate<'static>)>, DbClientError> {
        let krate = self
            .connection
            .query_row("SELECT id, name FROM crate WHERE name = ?1", [name], |res| {
                let id = res.get(0)?;
                // possibly redundant if name has already been given
                let name = res.get(1)?;

                Ok((id, name))
            })
            .optional()?
            .map(|(id, name)| (id, Crate { name }));

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
    fn insert_and_find_by_id() {
        let (_dir_guard, db) = DbClient::mock_db();

        let mock_record = mock_crate_record();

        let id = db.insert_crate_record(&mock_record).unwrap();

        let found_record = db.find_crate_by_id(id).unwrap().unwrap();
        assert_eq!(mock_record, found_record);

        let fake_id = mock_crate_id();
        let found_record = db.find_crate_by_id(fake_id).unwrap();
        assert!(found_record.is_none());
    }

    #[test]
    fn unique_name_index() {
        let (_dir_guard, db) = DbClient::mock_db();

        let mock_record = mock_crate_record();

        let first_insert_result = db.insert_crate_record(&mock_record);
        assert!(first_insert_result.is_ok());

        let second_insert_result = db.insert_crate_record(&mock_record);
        assert!(second_insert_result.is_err());
    }

    #[test]
    fn find_by_name() {
        let (_dir_guard, db) = DbClient::mock_db();

        let record = mock_crate_record();

        let id = db.insert_crate_record(&record).unwrap();

        let actual = db.find_crate_by_name(&record.name).unwrap().unwrap();
        let expected = (id, record);

        assert_eq!(expected, actual);
    }
}
