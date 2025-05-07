use crate::*;

statement_table!(WriteStatementId, WriteStatement<'_>, "write_register");

// Created when implementing Format
#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WriteStatement<'a> {
    // Often from derived implementations
    TypeStructure(TypeStructure),
    // Often from manual implementations
    #[serde(borrow)]
    Segments(Vec<Segment<'a>>),
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TypeStructure {
    name: String,
    variant: TypeStructureVariant,
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
    Named(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    statement_table_tests!(WriteStatement);

    impl StatementTableTest for WriteStatement<'_> {
        fn mock_id() -> Self::Id {
            WriteStatementId(ShortId(123))
        }

        fn mock() -> Self {
            WriteStatement::Segments(vec![Segment::Str("x".into())])
        }

        fn mock_other() -> Self {
            WriteStatement::Segments(vec![Segment::Str("y".into())])
        }
    }
}
