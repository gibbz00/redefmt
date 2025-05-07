// MainDb

mod krate;
pub(crate) use krate::{Crate, CrateId, CrateTable, CrateTableError};

// CrateDb
mod statement_table;
pub(crate) use statement_table::{StatementRegister, StatementTable, statement_table};
#[cfg(test)]
pub(crate) use statement_table::{StatementTableTest, insert_unchecked, statement_table_tests};
