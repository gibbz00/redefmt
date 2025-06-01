use redefmt_args::{deferred::DeferredFormatExpression, identifier::AnyIdentifier};
use redefmt_db::statement_table::stored_format_expression::StoredFormatExpression;

pub struct StatementUtils;

impl StatementUtils {
    pub fn prepare_stored<'a>(
        format_expression: DeferredFormatExpression<'a>,
        expressions_count: usize,
        provided_identifiers: Vec<AnyIdentifier<'a>>,
        append_newline: bool,
    ) -> StoredFormatExpression<'a> {
        StoredFormatExpression {
            expression: format_expression,
            append_newline,
            expected_positional_arg_count: expressions_count - provided_identifiers.len(),
            expected_named_args: provided_identifiers,
        }
    }
}
