mod sql_json {
    use rusqlite::{MappedRows, OptionalExtension, Result as SqlResult, Row};
    use serde::Deserialize;
    use serde_json::Value as JsonValue;

    use crate::*;

    pub trait SqlOptionalJsonExt {
        fn optional_json<'de, T: Deserialize<'de>>(self) -> Result<Option<T>, DbClientError>;
    }

    impl SqlOptionalJsonExt for SqlResult<JsonValue> {
        fn optional_json<'de, T: Deserialize<'de>>(self) -> Result<Option<T>, DbClientError> {
            self.optional()?.map(T::deserialize).transpose().map_err(Into::into)
        }
    }

    pub trait SqlListJsonExt<I> {
        fn list_json<'de, T: Deserialize<'de>>(self) -> Result<Vec<(I, T)>, DbClientError>;
    }

    impl<I, F> SqlListJsonExt<I> for MappedRows<'_, F>
    where
        F: FnMut(&Row<'_>) -> SqlResult<(I, JsonValue)>,
    {
        fn list_json<'de, T: Deserialize<'de>>(self) -> Result<Vec<(I, T)>, DbClientError> {
            self.map(|result| {
                result
                    .map_err(DbClientError::from)
                    .and_then(|(i, json)| T::deserialize(json).map(|t| (i, t)).map_err(Into::into))
            })
            .collect::<Result<_, _>>()
        }
    }
}
pub(crate) use sql_json::{SqlListJsonExt, SqlOptionalJsonExt};
