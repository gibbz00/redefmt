use redefmt_args::deferred::DeferredFormatExpression;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, derive_getters::Getters)]
pub struct StoredFormatExpression<'a> {
    #[serde(borrow)]
    expression: DeferredFormatExpression<'a>,
    append_newline: bool,
}

impl<'a> StoredFormatExpression<'a> {
    pub fn new(expression: DeferredFormatExpression<'a>, append_newline: bool) -> Self {
        Self { expression, append_newline }
    }
}
