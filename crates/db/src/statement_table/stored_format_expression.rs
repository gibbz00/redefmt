use redefmt_args::{deferred::DeferredFormatExpression, identifier::AnyIdentifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct StoredFormatExpression<'a> {
    #[serde(borrow)]
    pub expression: DeferredFormatExpression<'a>,
    pub append_newline: bool,
    pub expected_positional_arg_count: usize,
    // Order matters, technically unique.
    pub expected_named_args: Vec<AnyIdentifier<'a>>,
}
