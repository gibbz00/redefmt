use redefmt_args::MappedFormatExpression;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, derive_getters::Getters)]
pub struct StoredFormatExpression<'a> {
    #[serde(borrow)]
    expression: MappedFormatExpression<'a>,
    append_newline: bool,
}

impl<'a> StoredFormatExpression<'a> {
    pub fn new(expression: MappedFormatExpression<'a>, append_newline: bool) -> Self {
        Self { expression, append_newline }
    }
}
