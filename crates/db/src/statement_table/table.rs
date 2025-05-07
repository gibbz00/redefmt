use rusqlite::{ToSql, types::FromSql};
use serde::{Deserialize, Serialize};

use crate::*;

pub trait StatementTable: private::Sealed + PartialEq + std::hash::Hash + Serialize + Deserialize<'static> {
    type Id: ToSql + FromSql;
    const NAME: &'static str;
}

mod private {
    use crate::*;

    pub trait Sealed {}

    impl Sealed for WriteStatement<'_> {}
    impl Sealed for PrintStatement<'_> {}
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
