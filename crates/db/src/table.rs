mod krate {
    use crate::*;

    #[derive(Debug, Clone, Copy)]
    pub struct CrateId(u16);

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

    impl Crate<'_> {
        const TABLE_NAME: &'static str = "crate";
    }

    pub trait CrateTable {
        fn find_crate_record(&self, id: CrateId) -> Result<Option<Crate<'static>>, DbClientError>;

        fn create_crate_record(&self, record: &Crate<'_>) -> Result<CrateId, DbClientError>;
    }

    impl CrateTable for DbClient<MainDb> {
        fn find_crate_record(&self, id: CrateId) -> Result<Option<Crate<'static>>, DbClientError> {
            let mut statement = self.connection.prepare("SELECT name FROM crate WHERE id = ?1")?;
            let mut row_iter = statement.query_map([id.0], |res| res.get::<_, String>(0))?;

            // unique index on name should result in only one record being returned
            let Some(name_string) = row_iter.next().transpose()? else {
                return Ok(None);
            };

            let name = CrateName::new(name_string).map_err(CrateTableError::Name)?;

            Ok(Some(Crate { name }))
        }

        fn create_crate_record(&self, record: &Crate<'_>) -> Result<CrateId, DbClientError> {
            let long_id = self.connection.query_row_and_then(
                "INSERT INTO crate(name) VALUES (?1) RETURNING id",
                [record.name.as_ref()],
                |res| res.get::<_, i64>(0),
            )?;

            let inner_id = u16::try_from(long_id).map_err(|_| DbClientError::ExhaustedId(Crate::TABLE_NAME))?;

            Ok(CrateId(inner_id))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn mock_crate_id() -> CrateId {
            CrateId(123)
        }

        fn mock_crate_record() -> Crate<'static> {
            Crate { name: CrateName::new("x").unwrap() }
        }

        #[test]
        fn find_and_create() {
            let (_dir_guard, client) = DbClient::mock_db();

            let mock_record = mock_crate_record();

            let id = client.create_crate_record(&mock_record).unwrap();

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

            let first_insert_result = client.create_crate_record(&mock_record);
            assert!(first_insert_result.is_ok());

            let second_insert_result = client.create_crate_record(&mock_record);
            assert!(second_insert_result.is_err());
        }
    }
}
pub(crate) use krate::{Crate, CrateId, CrateTable, CrateTableError};
