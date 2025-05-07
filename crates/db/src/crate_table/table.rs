use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrateId(pub(crate) ShortId);

sql_newtype!(CrateId);

#[derive(Debug, PartialEq)]
pub struct Crate<'a> {
    pub(super) name: CrateName<'a>,
}
