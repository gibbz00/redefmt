short_id_newtype!(CrateId);

short_id_newtype!(PrintStatementId);

short_id_newtype!(WriteStatementId);

short_id_newtype!(TypeStructureId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShortId(pub(crate) u16);

impl core::fmt::Display for ShortId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "db")]
mod db {
    use rusqlite::{
        ToSql,
        types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
    };

    use super::*;

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
}

#[cfg(feature = "db")]
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
#[cfg(feature = "db")]
use sql_newtype;

macro_rules! short_id_newtype {
    ($id:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $id($crate::ShortId);

        impl ::core::fmt::Display for $id {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl $id {
            pub fn new(inner: u16) -> Self {
                Self(ShortId(inner))
            }
        }

        impl AsRef<u16> for $id {
            fn as_ref(&self) -> &u16 {
                &self.0.0
            }
        }

        #[cfg(feature = "db")]
        sql_newtype!($id);
    };
}
use short_id_newtype;
