use redefmt_args::provided_args::CombinedFormatString;
use redefmt_internal::identifiers::WriteStatementId;

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
    FormatString(CombinedFormatString<'a, 'a>),
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
    Named(Vec<String>),
}

#[cfg(test)]
mod tests {
    use redefmt_args::{FormatString, provided_args::ProvidedArgs};

    use super::*;

    statement_table_tests!(WriteStatement);

    impl StatementTableTest for WriteStatement<'_> {
        fn mock_id() -> Self::Id {
            WriteStatementId::new(123)
        }

        fn mock() -> Self {
            let combined = {
                let format_string = FormatString::parse("x").unwrap();
                let provided_args = ProvidedArgs::default();
                CombinedFormatString::combine(format_string, provided_args).unwrap()
            };

            WriteStatement::FormatString(combined)
        }

        fn mock_other() -> Self {
            let combined = {
                let format_string = FormatString::parse("y").unwrap();
                let provided_args = ProvidedArgs::default();
                CombinedFormatString::combine(format_string, provided_args).unwrap()
            };

            WriteStatement::FormatString(combined)
        }
    }
}
