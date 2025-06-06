use std::borrow::Cow;

use redefmt_core::identifiers::PrintStatementId;

use crate::*;

impl StatementTable for PrintStatement<'_> {
    type Id = PrintStatementId;

    const NAME: &'static str = "print_register";
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PrintStatement<'a> {
    pub location: Location<'a>,
    #[serde(borrow)]
    pub stored_expression: StoredFormatExpression<'a>,
}

/// Print statement call site location
///
/// Crate could be inferred the crate database itself.
///
/// Module path currently not saved since there isn't a reliable way to extract it from proc macros.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Location<'a> {
    pub file: Cow<'a, str>,
    pub line: u32,
}

#[cfg(test)]
mod tests {
    use redefmt_args::{processed_format_string, processor::ProcessedFormatString};

    use super::*;

    statement_table_tests!(PrintStatement);

    impl StatementTableTest for PrintStatement<'_> {
        fn mock_id() -> Self::Id {
            PrintStatementId::new(123)
        }

        fn mock() -> Self {
            PrintStatement {
                location: mock_location(),
                stored_expression: mock_stored_expression(processed_format_string!("x")),
            }
        }

        fn mock_other() -> Self {
            PrintStatement {
                location: mock_location(),
                stored_expression: mock_stored_expression(processed_format_string!("y")),
            }
        }
    }

    fn mock_location() -> Location<'static> {
        Location { file: "file.rs".into(), line: 1 }
    }

    fn mock_stored_expression(expression: ProcessedFormatString) -> StoredFormatExpression {
        StoredFormatExpression {
            format_string: expression,
            append_newline: false,
            expected_positional_arg_count: 0,
            expected_named_args: Default::default(),
        }
    }
}
