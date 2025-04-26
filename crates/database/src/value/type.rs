use alloc::{string::String, vec::Vec};

use crate::*;

pub enum Type {
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
    Tuple(Vec<TypeDocumentId>),
    Named(Vec<(String, TypeDocumentId)>),
}

pub struct Union {
    name: String,
    fields: Vec<(String, TypeDocumentId)>,
}

pub struct Enum {
    name: String,
    // sends index denoting variant over the wire
    fields: Vec<(String, EnumVariant)>,
}

pub enum EnumVariant {
    // nothing needs to be sent
    Unit,
    Tuple(Vec<TypeDocumentId>),
    Named(Vec<(String, TypeDocumentId)>),
}
