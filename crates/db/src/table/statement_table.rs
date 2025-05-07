use rusqlite::{ToSql, params, types::FromSql};
use serde::{Deserialize, Serialize};

use crate::*;

// TODO: seal?
pub trait StatementTable: PartialEq + std::hash::Hash + Serialize + Deserialize<'static> {
    type Id: ToSql + FromSql;

    const NAME: &'static str;
}

macro_rules! statement_table {
    ($id:ident, $statement:ty, $table_name:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $id($crate::ShortId);

        sql_newtype!($id);

        impl $crate::StatementTable for $statement {
            type Id = $id;

            const NAME: &'static str = $table_name;
        }
    };
}
pub(crate) use statement_table;

pub trait StatementRegister<T: StatementTable> {
    fn find_statement_by_id(&self, id: T::Id) -> Result<Option<T>, DbClientError>;

    /// Returns a list due to the possibility of hash collisions
    #[allow(clippy::type_complexity)]
    fn find_statement_by_hash(&self, hash: Hash) -> Result<Vec<(T::Id, T)>, DbClientError>;

    fn insert_statement(&self, statement: &T) -> Result<T::Id, DbClientError>;
}

impl<T: StatementTable> StatementRegister<T> for DbClient<CrateDb> {
    fn find_statement_by_id(&self, id: T::Id) -> Result<Option<T>, DbClientError> {
        let mut prepared_statement = self
            .connection
            .prepare(&format!("SELECT json(statement) FROM {} WHERE id = ?1", T::NAME))?;

        prepared_statement.query_row([id], |res| res.get(0)).optional_json()
    }

    fn find_statement_by_hash(&self, hash: Hash) -> Result<Vec<(T::Id, T)>, DbClientError> {
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

    fn insert_statement(&self, statement: &T) -> Result<T::Id, DbClientError> {
        let hash = Hash::new(statement);

        let current_write_registers = <Self as StatementRegister<T>>::find_statement_by_hash(self, hash)?;

        for (current_id, current_statement) in current_write_registers {
            if &current_statement == statement {
                return Ok(current_id);
            }
        }

        insert_unchecked::<T>(self, hash, statement)
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

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    statement_table_tests!(MockStatement);

    statement_table!(MockRegisterId, MockStatement<'_>, "mock_register");

    #[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    pub struct MockStatement<'a> {
        value: Cow<'a, str>,
    }

    impl StatementTableTest for MockStatement<'static> {
        fn mock_id() -> Self::Id {
            MockRegisterId(ShortId(123))
        }

        fn mock() -> Self {
            MockStatement { value: "x".into() }
        }

        fn mock_other() -> Self {
            MockStatement { value: "y".into() }
        }

        fn init_schema(db: &DbClient<CrateDb>) {
            const SCHEMA: &str = "
                CREATE TABLE mock_register(
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    hash INTEGER NOT NULL,
                    statement BLOB NOT NULL
                );

                CREATE INDEX mock_hash ON mock_register(hash);
            ";

            db.connection.execute_batch(SCHEMA).unwrap();
        }
    }
}

#[cfg(test)]
mod gen_tests {
    use super::*;

    pub trait StatementTableTest: StatementTable {
        fn mock_id() -> Self::Id;
        fn mock() -> Self;
        fn mock_other() -> Self;
        fn init_schema(_db: &DbClient<CrateDb>) {}
    }

    macro_rules! statement_table_tests {
        ($statement_table:ident) => {
            mod statement_table {
                use super::*;

                #[test]
                fn find_none_by_id() {
                    let (_dir_guard, db) = DbClient::mock_db();
                    $statement_table::init_schema(&db);

                    let mock_id = $statement_table::mock_id();

                    let record = find_helper(&db, mock_id);

                    assert!(record.is_none());
                }

                #[test]
                fn insert_and_find_by_id() {
                    let (_dir_guard, db) = DbClient::mock_db();
                    $statement_table::init_schema(&db);

                    let statement = $statement_table::mock();

                    let inserted_id = insert_helper(&db, &statement);

                    let returned_statement = find_helper(&db, inserted_id).unwrap();

                    assert_eq!(statement, returned_statement);
                }

                #[test]
                fn insert_cached() {
                    let (_dir_guard, db) = DbClient::mock_db();
                    $statement_table::init_schema(&db);

                    let statement = $statement_table::mock();
                    let first_id = insert_helper(&db, &statement);
                    let second_id = insert_helper(&db, &statement);

                    assert_eq!(first_id, second_id);

                    let other_statement = $statement_table::mock_other();
                    let first_id_other = insert_helper(&db, &other_statement);
                    let second_id_other = insert_helper(&db, &other_statement);

                    assert_ne!(first_id, first_id_other);
                    assert_eq!(first_id_other, second_id_other);
                }

                #[test]
                fn insert_collisioned() {
                    let (_dir_guard, db) = DbClient::mock_db();
                    $statement_table::init_schema(&db);

                    let statement = $statement_table::mock();
                    let statement_other = $statement_table::mock_other();

                    let statement_hash = Hash::new(&statement);

                    let other_id = insert_unchecked(&db, statement_hash, &statement_other).unwrap();

                    let first_id = insert_helper(&db, &statement);

                    // inserts even if hash exists, but where the content differs
                    assert_ne!(other_id, first_id);

                    // does not insert if hash exists, and where content exists
                    let second_id = insert_helper(&db, &statement);

                    assert_eq!(first_id, second_id);
                }

                #[test]
                fn find_by_hash() {
                    let (_dir_guard, db) = DbClient::mock_db();
                    $statement_table::init_schema(&db);

                    let statement = $statement_table::mock();
                    let statement_other = $statement_table::mock_other();

                    let statement_hash = Hash::new(&statement);
                    let statement_hash_other = Hash::new(&statement_other);

                    let id = insert_unchecked(&db, statement_hash, &statement).unwrap();
                    let other_id = insert_unchecked(&db, statement_hash, &statement_other).unwrap();

                    assert_ne!(id, other_id);

                    // to assert that all records aren't simply returned
                    let _ = insert_unchecked(&db, statement_hash_other, &statement_other).unwrap();

                    let mut actual_records = db.find_statement_by_hash(statement_hash).unwrap();
                    actual_records.sort_unstable_by_key(|(id, _)| *id);

                    let expected_records = vec![(id, statement), (other_id, statement_other)];

                    assert_eq!(expected_records, actual_records);
                }

                fn find_helper(
                    db: &DbClient<CrateDb>,
                    id: <$statement_table as StatementTable>::Id,
                ) -> Option<$statement_table<'static>> {
                    db.find_statement_by_id(id).unwrap()
                }

                fn insert_helper<'a>(
                    db: &DbClient<CrateDb>,
                    statement: &$statement_table<'_>,
                ) -> <$statement_table<'a> as StatementTable>::Id {
                    db.insert_statement(statement).unwrap()
                }
            }
        };
    }
    pub(crate) use statement_table_tests;
}
#[cfg(test)]
pub(crate) use gen_tests::{StatementTableTest, statement_table_tests};
