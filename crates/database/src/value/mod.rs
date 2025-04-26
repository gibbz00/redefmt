use crate::*;

mod core;
pub(crate) use core::NormalizedValue;

mod primitive;
pub(crate) use primitive::Primitive;

mod collection;
pub(crate) use collection::*;

mod type_tree {
    use crate::*;

    // ID: TypeID
    pub enum NormalizedType {
        Struct(Struct),
        Enum(Enum),
        Union(Union),
    }

    pub struct Struct {
        name: String,
        variant: StructVariant,
    }

    pub enum StructVariant {
        // nothing needs to be written
        Unit,
        Tuple(Vec<NormalizedValue>),
        Named(Vec<(String, NormalizedValue)>),
    }

    pub struct Union {
        name: String,
        fields: Vec<(String, NormalizedValue)>,
    }

    pub struct Enum {
        name: String,
        // sends index denoting variant over the wire
        fields: Vec<(String, EnumVariant)>,
    }

    pub enum EnumVariant {
        // nothing needs to be sent
        Unit,
        Tuple(Vec<NormalizedValue>),
        Named(Vec<(String, NormalizedValue)>),
    }
}
pub(crate) use type_tree::*;
