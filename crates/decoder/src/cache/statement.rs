use std::hash::Hash;

use elsa::sync::FrozenMap;
use redefmt_core::identifiers::CrateId;
use redefmt_db::{Table, statement_table::StatementTable};

use crate::*;

type InnerMap<T> = FrozenMap<(CrateId, <T as StatementTable>::Id), Box<T>>;

pub struct StatementCache<T: StatementTable> {
    map: InnerMap<T>,
}

impl<T: StatementTable> StatementCache<T> {
    pub fn get_or_insert(&self, id: T::Id, crate_context: CrateContext) -> Result<&T, RedefmtDecoderError>
    where
        T::Id: Copy + AsRef<u16> + Hash + Eq,
    {
        let statement_id = (crate_context.id, id);

        let statement = match self.map.get(&statement_id) {
            Some(statement) => statement,
            None => {
                let Some(statement) = crate_context.db.find_by_id(id)? else {
                    return Err(RedefmtDecoderError::UnknownStatement(
                        *id.as_ref(),
                        T::NAME,
                        crate_context.record.name.clone(),
                    ));
                };

                self.map.insert(statement_id, Box::new(statement))
            }
        };

        Ok(statement)
    }
}

// overkill to pull in impl_tools::auto_impl
impl<T: StatementTable> Default for StatementCache<T> {
    fn default() -> Self {
        Self { map: Default::default() }
    }
}

#[cfg(test)]
mod ext {
    use super::*;

    impl<T: StatementTable> StatementCache<T> {
        pub fn inner(&self) -> &InnerMap<T> {
            &self.map
        }
    }
}
