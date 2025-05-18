// https://github.com/rusqlite/rusqlite/issues/1436
macro_rules! sql_newtype {
    ($newtype:ty) => {
        impl rusqlite::ToSql for $newtype {
            fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
                self.0.to_sql()
            }
        }

        impl rusqlite::types::FromSql for $newtype {
            fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
                rusqlite::types::FromSql::column_result(value).map(Self)
            }
        }
    };
}
pub(crate) use sql_newtype;
