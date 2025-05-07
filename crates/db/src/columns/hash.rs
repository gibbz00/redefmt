use std::hash::{DefaultHasher, Hash as StdHash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(transparent)]
pub struct Hash(u64);

impl Hash {
    pub fn new(object: &impl StdHash) -> Self {
        let mut hasher = DefaultHasher::new();
        object.hash(&mut hasher);
        let inner = hasher.finish();
        Hash(inner)
    }
}

/// SQLite uses i64 exclusively. To the u64 output of `Hasher::finish`,
/// we transmute the bytes to and from the signed counterpart.
mod sql {
    use rusqlite::{
        ToSql,
        types::{FromSql, FromSqlResult, ToSqlOutput, Value, ValueRef},
    };

    use super::*;

    impl ToSql for Hash {
        fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
            let int = i64::from_ne_bytes(self.0.to_ne_bytes());
            Ok(ToSqlOutput::Owned(Value::Integer(int)))
        }
    }

    impl FromSql for Hash {
        fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
            let int = value.as_i64()?;
            let uint = u64::from_ne_bytes(int.to_ne_bytes());
            Ok(Hash(uint))
        }
    }

    #[cfg(test)]
    mod tests {
        use rusqlite::Connection;

        use super::*;

        #[test]
        fn to_and_from_sql() {
            let connection = Connection::open_in_memory().unwrap();

            connection
                .execute("CREATE TABLE foo(hash INTEGER NOT NULL);", [])
                .unwrap();

            let hash = Hash(123);

            let id = connection
                .query_row_and_then("INSERT INTO foo (hash) VALUES (?1) RETURNING rowid", [hash], |res| {
                    res.get::<_, i64>(0)
                })
                .unwrap();

            let returned_hash = connection
                .query_row_and_then("select hash from foo where rowid = ?1", [id], |res| {
                    res.get::<_, Hash>(0)
                })
                .unwrap();

            assert_eq!(hash, returned_hash);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Hash)]
    struct X(u64);

    #[test]
    fn hash() {
        let first = Hash::new(&X(1));
        let second = Hash::new(&X(1));
        let third = Hash::new(&X(2));

        assert_eq!(first, second);
        assert_ne!(second, third);
    }
}
