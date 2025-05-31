use redefmt_args::deferred::DeferredFormatExpression;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum PrintCounterConfigError {
    #[error("counter format string expects one and only one argument")]
    InvalidExpectedArgCount,
}

#[derive(Debug)]
pub struct PrintCounterConfig {
    format_expression: DeferredFormatExpression<'static>,
}

impl PrintCounterConfig {
    pub fn new(format_expression: DeferredFormatExpression<'static>) -> Result<Self, PrintCounterConfigError> {
        if format_expression.expected_arg_count() != 1 {
            return Err(PrintCounterConfigError::InvalidExpectedArgCount);
        }

        Ok(PrintCounterConfig { format_expression })
    }
}
