use rusqlite::{ToSql, params, types::FromSql};
use serde::{Deserialize, Serialize};

use crate::*;

pub trait StatementTable: private::Sealed + PartialEq + std::hash::Hash + Serialize + Deserialize<'static> {
    type Id: ToSql + FromSql;
    const NAME: &'static str;
}

impl<T: StatementTable> Record for T {
    type Id = T::Id;
}

mod private {
    use crate::*;

    pub trait Sealed {}

    impl Sealed for WriteStatement<'_> {}
    impl Sealed for PrintStatement<'_> {}
}

impl<T: StatementTable> Table<T> for DbClient<CrateDb> {
    fn find_by_id(&self, id: <T as Record>::Id) -> Result<Option<T>, DbClientError> {
        let mut prepared_statement = self
            .connection
            .prepare(&format!("SELECT json(statement) FROM {} WHERE id = ?1", T::NAME))?;

        prepared_statement.query_row([id], |res| res.get(0)).optional_json()
    }

    fn insert(&self, statement: &T) -> Result<<T as Record>::Id, DbClientError> {
        let hash = Hash::new(statement);

        let current_write_registers = self.find_statement_by_hash::<T>(hash)?;

        for (current_id, current_statement) in current_write_registers {
            if &current_statement == statement {
                return Ok(current_id);
            }
        }

        insert_unchecked::<T>(self, hash, statement)
    }
}

// Separate trait to avoid exposing `Hash` in public API
pub trait StatementDbClientInner {
    /// Returns a list due to the possibility of hash collisions
    #[allow(clippy::type_complexity)]
    fn find_statement_by_hash<T: StatementTable>(&self, hash: Hash) -> Result<Vec<(T::Id, T)>, DbClientError>;
}

impl StatementDbClientInner for DbClient<CrateDb> {
    fn find_statement_by_hash<T: StatementTable>(&self, hash: Hash) -> Result<Vec<(T::Id, T)>, DbClientError> {
        let mut prepared_statement = self
            .connection
            .prepare(&format!("SELECT id, json(statement) FROM {} WHERE hash = ?1", T::NAME))?;

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
}

// separated to simplify testing of simulated hash collisions
pub(crate) fn insert_unchecked<T: StatementTable>(
    db: &DbClient<CrateDb>,
    hash: Hash,
    statement: &T,
) -> Result<T::Id, DbClientError> {
    let json_statement = serde_json::to_value(statement)?;

    let mut prepared_statement = db.connection.prepare(&format!(
        "INSERT INTO {} (hash, statement) VALUES (?1, jsonb(?2)) RETURNING id",
        T::NAME,
    ))?;

    prepared_statement
        .query_row(params![hash, json_statement], |res| res.get(0))
        .map_err(Into::into)
}
