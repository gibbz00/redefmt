use std::borrow::Cow;

#[derive(Debug, derive_more::Deref)]
pub struct CrateName<'a>(Cow<'a, str>);

#[derive(Debug, thiserror::Error)]
pub enum CrateNameError {
    #[error("'{0}' not a valid crate name character")]
    InvalidChar(char),
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
                return Err(CrateNameError::InvalidChar(char));
            }
        }

        Ok(Self(cow_str))
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
            assert!(result.is_ok(), "initial str: {}", str)
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
                CrateNameError::InvalidChar(char) => {
                    assert_eq!(invalid_char, char, "initial str: {}", str)
                }
            }
        }
    }
}
