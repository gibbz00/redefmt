use crate::*;

statement_table!(WriteRegisterId, WriteStatement<'_>, "write_register");

// Created when implementing Format
#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WriteStatement<'a> {
    // Often from derived implementations
    TypeStructure(TypeStructure),
    // Often from manual implementations
    #[serde(borrow)]
    Segments(Vec<Segment<'a>>),
}

#[cfg(test)]
mod tests {
    use super::*;

    statement_table_tests!(WriteStatement);

    impl StatementTableTest for WriteStatement<'_> {
        fn mock_id() -> Self::Id {
            WriteRegisterId(ShortId(123))
        }

        fn mock() -> Self {
            WriteStatement::Segments(vec![Segment::Str("x".into())])
        }

        fn mock_other() -> Self {
            WriteStatement::Segments(vec![Segment::Str("y".into())])
        }
    }
}
