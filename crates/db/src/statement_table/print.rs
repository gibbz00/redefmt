use std::borrow::Cow;

use redefmt_core::identifiers::PrintStatementId;

use crate::*;

impl StatementTable for PrintStatement<'_> {
    type Id = PrintStatementId;

    const NAME: &'static str = "print_register";
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, derive_getters::Getters)]
pub struct PrintStatement<'a> {
    location: Location<'a>,
    #[serde(borrow)]
    format_expression: StoredFormatExpression<'a>,
}

impl<'a> PrintStatement<'a> {
    pub fn new(location: Location<'a>, format_expression: StoredFormatExpression<'a>) -> Self {
        Self { location, format_expression }
    }
}

/// Print statement call site location
///
/// Crate could be inferred the crate database itself.
///
/// Module path currently not saved since there isn't a reliable way to extract it from proc macros.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, derive_getters::Getters)]
pub struct Location<'a> {
    file: Cow<'a, str>,
    line: u32,
}

impl<'a> Location<'a> {
    pub fn new(file: impl Into<Cow<'a, str>>, line: u32) -> Self {
        Self { file: file.into(), line }
    }
}

#[cfg(test)]
mod tests {
    use redefmt_args::mapped_format_expression;

    use super::*;

    statement_table_tests!(PrintStatement);

    impl StatementTableTest for PrintStatement<'_> {
        fn mock_id() -> Self::Id {
            PrintStatementId::new(123)
        }

        fn mock() -> Self {
            PrintStatement::new(
                mock_location(),
                StoredFormatExpression::new(mapped_format_expression!("x"), false),
            )
        }

        fn mock_other() -> Self {
            PrintStatement::new(
                mock_location(),
                StoredFormatExpression::new(mapped_format_expression!("y"), false),
            )
        }
    }

    fn mock_location() -> Location<'static> {
        Location { file: "file.rs".into(), line: 1 }
    }
}
