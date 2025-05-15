use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use redefmt_db::{Table, statement_table::StatementTable};

use crate::*;

type InnerMap<T> = HashMap<<T as StatementTable>::Id, T>;

pub struct StatementCache<T: StatementTable> {
    map: InnerMap<T>,
}

// Deriving Default sets default bounds on T. Pulling in impl_tools::auto_impl
// is overkill for only one derive.
impl<T: StatementTable> Default for StatementCache<T> {
    fn default() -> Self {
        Self { map: Default::default() }
    }
}

impl<T: StatementTable> StatementCache<T> {
    pub fn fetch_and_save(
        &mut self,
        id: T::Id,
        (crate_db, crate_record): &CrateValue,
    ) -> Result<&T, RedefmtDecoderError>
    where
        T::Id: Copy + AsRef<u16> + Hash + Eq,
    {
        if let Entry::Vacant(entry) = self.map.entry(id) {
            let Some(statement) = crate_db.find_by_id(id)? else {
                return Err(RedefmtDecoderError::UnknownStatement(
                    *id.as_ref(),
                    T::NAME,
                    crate_record.name().clone(),
                ));
            };

            entry.insert(statement);
        }

        let statement = self.map.get(&id).expect("statement not cached");

        Ok(statement)
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
