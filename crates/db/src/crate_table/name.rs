use std::borrow::Cow;

use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};

#[derive(Debug, Clone, PartialEq, derive_more::Deref, derive_more::Display)]
pub struct CrateName<'a>(Cow<'a, str>);

#[derive(Debug, thiserror::Error)]
pub enum CrateNameError {
    #[error("'{0}' not a valid crate name character in '{1}'")]
    InvalidChar(char, Cow<'static, str>),
}

impl<'a> CrateName<'a> {
    /// Enforces Cargo's [name restrictions]
    ///
    /// In short, only alphanumerics, underscores, and dashes are allowed.
    ///
    /// Primarily as a means to mitigate unauthorized path traversals since it
    /// is used as the directory name its database path.
    ///
    /// [name restrictions]: https://doc.rust-lang.org/cargo/reference/registry-index.html#name-restrictions
    pub fn new(cow_str: impl Into<Cow<'a, str>>) -> Result<Self, CrateNameError> {
        let cow_str = cow_str.into();

        for char in cow_str.chars() {
            if !['_', '-'].contains(&char) && !char.is_ascii_alphanumeric() {
                return Err(CrateNameError::InvalidChar(char, Cow::Owned(cow_str.into_owned())));
            }
        }

        Ok(Self(cow_str))
    }
}

impl<'a> FromSql for CrateName<'a> {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let str = value
            .as_str()?
            // unfortunate allocation, think it's a result of FromSql trait
            // signature not allowing lifetime bounds on ValueRef
            .to_owned();

        Self::new(str).map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

impl<'a> ToSql for CrateName<'a> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        assert_ok("abc");
        assert_ok("abc-def");
        assert_ok("abc_def");

        fn assert_ok(str: &str) {
            let result = CrateName::new(str);
            assert!(result.is_ok(), "initial str: {str}")
        }
    }

    #[test]
    fn invalid() {
        assert_invalid("x/x", '/');
        assert_invalid("../../x", '.');
        assert_invalid("x\\x", '\\');
        assert_invalid("x\n", '\n');

        fn assert_invalid(str: &str, invalid_char: char) {
            let err = CrateName::new(str).unwrap_err();

            match err {
                CrateNameError::InvalidChar(char, _) => {
                    assert_eq!(invalid_char, char, "initial str: {str}")
                }
            }
        }
    }
}
