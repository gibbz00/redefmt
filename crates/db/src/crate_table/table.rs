use crate::*;

short_id_newtype!(CrateId);

#[derive(Debug, PartialEq, derive_getters::Getters)]
pub struct Crate<'a> {
    pub(super) name: CrateName<'a>,
}

impl<'a> Crate<'a> {
    pub fn new(name: CrateName<'a>) -> Self {
        Self { name }
    }
}
