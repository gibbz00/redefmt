use rusqlite::params;

use crate::*;

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

        let json_statement = serde_json::to_value(statement)?;

        self.connection
            .query_row_and_then(
                "INSERT INTO write_register(hash, jsonb(statement)) VALUES (?1, ?2) RETURNING id",
                params![hash, json_statement],
                |res| res.get(0),
            )
            .map_err(Into::into)
    }
}
