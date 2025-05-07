use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShortId(pub(crate) u16);

impl ToSql for ShortId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let int = self.0 as i64;
        Ok(ToSqlOutput::Owned(Value::Integer(int)))
    }
}

impl FromSql for ShortId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let int = value.as_i64()?;

        u16::try_from(int)
            .map(ShortId)
            .map_err(|_| FromSqlError::OutOfRange(int))
    }
}
