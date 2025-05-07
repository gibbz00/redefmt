use crate::*;

// Created when implementing Format
#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WriteStatement<'a> {
    // Often from derived implementations
    TypeStructure(TypeStructure),
    // Often from manual implementations
    #[serde(borrow)]
    Segments(Vec<Segment<'a>>),
}

impl<'a> WriteStatement<'a> {}
