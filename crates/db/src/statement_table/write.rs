use redefmt_core::identifiers::WriteStatementId;

use crate::*;

impl StatementTable for WriteStatement<'_> {
    type Id = WriteStatementId;

    const NAME: &'static str = "write_register";
}

// Created when implementing Format
#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WriteStatement<'a> {
    // Often from derived implementations
    TypeStructure(TypeStructure),
    // Often from manual implementations
    #[serde(borrow)]
    FormatExpression(StoredFormatExpression<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TypeStructure {
    pub name: String,
    pub variant: TypeStructureVariant,
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TypeStructureVariant {
    Struct(StructVariant),
    Enum(Vec<(String, StructVariant)>),
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StructVariant {
    Unit,
    /// Inner u8 represents field count
    Tuple(u8),
    /// List of field names
    Named(Vec<String>),
}

#[cfg(test)]
mod tests {
    use redefmt_args::deferred_format_expression;

    use super::*;

    statement_table_tests!(WriteStatement);

    impl StatementTableTest for WriteStatement<'_> {
        fn mock_id() -> Self::Id {
            WriteStatementId::new(123)
        }

        fn mock() -> Self {
            WriteStatement::FormatExpression(StoredFormatExpression::new(deferred_format_expression!("x"), false))
        }

        fn mock_other() -> Self {
            WriteStatement::FormatExpression(StoredFormatExpression::new(deferred_format_expression!("y"), false))
        }
    }
}
