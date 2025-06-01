use redefmt_args::FormatExpression;
use redefmt_db::statement_table::stored_format_expression::StoredFormatExpression;

pub struct StatementUtils;

impl StatementUtils {
    pub fn prepare_stored(format_expression: FormatExpression, append_newline: bool) -> StoredFormatExpression {
        let (deferred_format_expression, provided_args) = format_expression.defer();

        StoredFormatExpression {
            expression: deferred_format_expression,
            append_newline,
            expected_positional_arg_count: provided_args.positional.len(),
            expected_named_args: provided_args
                .named
                .into_iter()
                .map(|(identifier, _)| identifier)
                .collect(),
        }
    }
}
