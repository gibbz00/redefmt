use rusqlite::params;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WriteRegisterId(ShortId);

sql_newtype!(WriteRegisterId);

pub trait WriteRegisterTable {
    fn find_write_statement_by_id(&self, id: WriteRegisterId) -> Result<Option<WriteStatement>, DbClientError>;

    /// Returns a list due to the possibility of hash collisions
    fn find_write_statement_by_hash(&self, hash: Hash)
    -> Result<Vec<(WriteRegisterId, WriteStatement)>, DbClientError>;

    fn insert_write_statement(&self, statement: &WriteStatement) -> Result<WriteRegisterId, DbClientError>;
}

impl WriteRegisterTable for DbClient<CrateDb> {
    fn find_write_statement_by_id(&self, id: WriteRegisterId) -> Result<Option<WriteStatement>, DbClientError> {
        self.connection
            .query_row(
                "SELECT json(statement) FROM write_register WHERE id = ?1",
                [id],
                |res| res.get(0),
            )
            .optional_json()
    }

    fn find_write_statement_by_hash(
        &self,
        hash: Hash,
    ) -> Result<Vec<(WriteRegisterId, WriteStatement)>, DbClientError> {
        let mut prepared_statement = self
            .connection
            .prepare("SELECT id, json(statement) FROM write_register WHERE hash = ?1")?;

        let rows = prepared_statement
            .query_map([hash], |res| {
                let id = res.get(0)?;
                let json = res.get(1)?;

                Ok((id, json))
            })?
            .list_json()?;

        let mut records = Vec::with_capacity(rows.len());

        for (id, statement) in rows {
            records.push((id, statement))
        }

        Ok(records)
    }

    fn insert_write_statement(&self, statement: &WriteStatement) -> Result<WriteRegisterId, DbClientError> {
        let hash = Hash::new(statement);

        let current_write_registers = self.find_write_statement_by_hash(hash)?;

        for (current_id, current_statement) in current_write_registers {
            if &current_statement == statement {
                return Ok(current_id);
            }
        }

        insert_unchecked(self, hash, statement)
    }
}

// separated to simplify testing of simulated hash collisions
fn insert_unchecked(
    db: &DbClient<CrateDb>,
    hash: Hash,
    statement: &WriteStatement,
) -> Result<WriteRegisterId, DbClientError> {
    let json_statement = serde_json::to_value(statement)?;

    db.connection
        .query_row_and_then(
            "INSERT INTO write_register (hash, statement) VALUES (?1, jsonb(?2)) RETURNING id",
            params![hash, json_statement],
            |res| res.get(0),
        )
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_id() -> WriteRegisterId {
        WriteRegisterId(ShortId(123))
    }

    fn mock_statement() -> WriteStatement<'static> {
        WriteStatement::Segments(vec![Segment::Str("x".into())])
    }

    fn mock_statement_other() -> WriteStatement<'static> {
        WriteStatement::Segments(vec![Segment::Str("y".into())])
    }

    #[test]
    fn find_none_by_id() {
        let (_dir_guard, db) = DbClient::mock_db();

        let mock_id = mock_id();

        let record = db.find_write_statement_by_id(mock_id).unwrap();

        assert!(record.is_none());
    }

    #[test]
    fn insert_and_find_by_id() {
        let (_dir_guard, db) = DbClient::mock_db();

        let statement = mock_statement();

        let inserted_id = db.insert_write_statement(&statement).unwrap();

        let returned_statement = db.find_write_statement_by_id(inserted_id).unwrap().unwrap();

        assert_eq!(statement, returned_statement);
    }

    #[test]
    fn insert_cached() {
        let (_dir_guard, db) = DbClient::mock_db();

        let statement = mock_statement();
        let first_id = db.insert_write_statement(&statement).unwrap();
        let second_id = db.insert_write_statement(&statement).unwrap();

        assert_eq!(first_id, second_id);

        let other_statement = mock_statement_other();
        let first_id_other = db.insert_write_statement(&other_statement).unwrap();
        let second_id_other = db.insert_write_statement(&other_statement).unwrap();

        assert_ne!(first_id, first_id_other);
        assert_eq!(first_id_other, second_id_other);
    }

    #[test]
    fn insert_collisioned() {
        let (_dir_guard, db) = DbClient::mock_db();

        let statement = mock_statement();
        let statement_other = mock_statement_other();

        let statement_hash = Hash::new(&statement);

        let other_id = insert_unchecked(&db, statement_hash, &statement_other).unwrap();

        let first_id = db.insert_write_statement(&statement).unwrap();

        // inserts even if hash exists, but where the content differs
        assert_ne!(other_id, first_id);

        // does not insert if hash exists, and where content exists
        let second_id = db.insert_write_statement(&statement).unwrap();

        assert_eq!(first_id, second_id);
    }

    #[test]
    fn find_by_hash() {
        let (_dir_guard, db) = DbClient::mock_db();

        let statement = mock_statement();
        let statement_other = mock_statement_other();

        let statement_hash = Hash::new(&statement);
        let statement_hash_other = Hash::new(&statement_other);

        let id = insert_unchecked(&db, statement_hash, &statement).unwrap();
        let other_id = insert_unchecked(&db, statement_hash, &statement_other).unwrap();

        assert_ne!(id, other_id);

        // to assert that all records aren't simply returned
        let _ = insert_unchecked(&db, statement_hash_other, &statement_other).unwrap();

        let mut actual_records = db.find_write_statement_by_hash(statement_hash).unwrap();
        actual_records.sort_unstable_by_key(|(id, _)| *id);

        let expected_records = vec![(id, statement), (other_id, statement_other)];

        assert_eq!(expected_records, actual_records);
    }
}
