use std::borrow::Cow;

use redefmt_core::identifiers::TypeStructureId;

use crate::*;

impl<'a> StatementTable for TypeStructure<'a> {
    type Id = TypeStructureId;

    const NAME: &'static str = "type_structure_register";
}

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TypeStructure<'a> {
    pub name: Cow<'a, str>,
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
    use super::*;

    statement_table_tests!(TypeStructure);

    impl StatementTableTest for TypeStructure<'_> {
        fn mock_id() -> Self::Id {
            TypeStructureId::new(123)
        }

        fn mock() -> Self {
            mock_type_structure_expression("Foo")
        }

        fn mock_other() -> Self {
            mock_type_structure_expression("Bar")
        }
    }

    fn mock_type_structure_expression(name: &str) -> TypeStructure<'_> {
        TypeStructure {
            name: name.into(),
            variant: TypeStructureVariant::Struct(StructVariant::Unit),
        }
    }
}
