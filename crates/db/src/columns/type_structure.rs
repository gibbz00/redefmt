#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct TypeStructure {
    name: String,
    variant: TypeStructureVariant,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum TypeStructureVariant {
    Struct(StructVariant),
    Enum(Vec<(String, StructVariant)>),
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum StructVariant {
    Unit,
    /// Inner u8 represents field count
    Tuple(u8),
    Named(Vec<String>),
}
