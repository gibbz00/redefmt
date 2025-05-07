use crate::*;

pub trait StatementTableTest: StatementTable {
    fn mock_id() -> Self::Id;
    fn mock() -> Self;
    fn mock_other() -> Self;
}

macro_rules! statement_table_tests {
    ($statement_table:ident) => {
        mod statement_table {
            use super::*;

            #[test]
            fn find_none_by_id() {
                let (_dir_guard, db) = DbClient::mock_db();

                let mock_id = $statement_table::mock_id();

                let record = find_helper(&db, mock_id);

                assert!(record.is_none());
            }

            #[test]
            fn insert_and_find_by_id() {
                let (_dir_guard, db) = DbClient::mock_db();

                let statement = $statement_table::mock();

                let inserted_id = insert_helper(&db, &statement);

                let returned_statement = find_helper(&db, inserted_id).unwrap();

                assert_eq!(statement, returned_statement);
            }

            #[test]
            fn insert_cached() {
                let (_dir_guard, db) = DbClient::mock_db();

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
