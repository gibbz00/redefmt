use redefmt_args::{deferred::DeferredFormatExpression, identifier::AnyIdentifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct StoredFormatExpression<'a> {
    #[serde(borrow)]
    pub expression: DeferredFormatExpression<'a>,
    pub append_newline: bool,
    pub expected_positional_arg_count: usize,
    // Technically unique
    pub expected_named_args: Vec<AnyIdentifier<'a>>,
}

impl StoredFormatExpression<'_> {
    pub fn expected_arg_count(&self) -> usize {
        self.expected_positional_arg_count + self.expected_named_args.len()
    }
}
