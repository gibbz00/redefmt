use redefmt_core::identifiers::WriteStatementId;

use crate::*;

impl StatementTable for WriteStatement<'_> {
    type Id = WriteStatementId;

    const NAME: &'static str = "write_register";
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct WriteStatement<'a>(#[serde(borrow)] pub StoredFormatExpression<'a>);

#[cfg(test)]
mod tests {
    use redefmt_args::{processed_format_string, processor::ProcessedFormatString};

    use super::*;

    statement_table_tests!(WriteStatement);

    impl StatementTableTest for WriteStatement<'_> {
        fn mock_id() -> Self::Id {
            WriteStatementId::new(123)
        }

        fn mock() -> Self {
            WriteStatement(mock_stored_expression(processed_format_string!("x")))
        }

        fn mock_other() -> Self {
            WriteStatement(mock_stored_expression(processed_format_string!("y")))
        }
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
