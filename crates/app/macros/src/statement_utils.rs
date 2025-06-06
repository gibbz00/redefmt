use redefmt_args::FormatExpression;
use redefmt_db::statement_table::stored_format_expression::StoredFormatExpression;

pub struct StatementUtils;

impl StatementUtils {
    pub fn dissolve_expression<'a>(
        format_expression: FormatExpression<'a>,
        append_newline: bool,
    ) -> (StoredFormatExpression<'a>, Vec<syn::Expr>) {
        let FormatExpression { processed_format_string, provided_args } = format_expression;

        let mut provided_expressions = provided_args.positional.into_iter().collect::<Vec<_>>();

        let provided_identifiers = provided_args
            .named
            .into_iter()
            .map(|(identifier, expr)| {
                provided_expressions.push(expr);
                identifier
            })
            .collect::<Vec<_>>();

        let stored_expression = StoredFormatExpression {
            format_string: processed_format_string,
            append_newline,
            expected_positional_arg_count: provided_expressions.len() - provided_identifiers.len(),
            expected_named_args: provided_identifiers,
        };

        (stored_expression, provided_expressions)
    }
}
