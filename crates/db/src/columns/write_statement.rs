use crate::*;

// Created when implementing Format
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum WriteStatement<'a> {
    // Often from derived implementations
    TypeStructure(TypeStructure),
    // Often from manual implementations
    #[serde(borrow)]
    Segments(Vec<Segment<'a>>),
}

impl<'a> WriteStatement<'a> {}
