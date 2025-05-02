// ID: defmt::common::WriteContentId;

use redefmt_args::FormatOptions;

// Created when implementing Format
pub enum WriteContent {
    // Often from derived implementations
    TypeStructure(TypeStructure),
    // Often from manual implementations
    Segments(Vec<Segment>),
}

pub enum Segment {
    String(String),
    Arg(FormatOptions<'static>),
}

pub struct TypeStructure {
    name: String,
    variant: TypeStructureVariant,
}

pub enum TypeStructureVariant {
    Struct(StructVariant),
    Enum(Vec<(String, StructVariant)>),
}

pub enum StructVariant {
    Unit,
    Tuple { field_count: u8 },
    Named(Vec<String>),
}
