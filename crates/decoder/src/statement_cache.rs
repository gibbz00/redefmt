use std::hash::Hash;

use elsa::sync::FrozenMap;
use redefmt_db::{Table, statement_table::StatementTable};

use crate::*;

type InnerMap<T> = FrozenMap<<T as StatementTable>::Id, Box<T>>;

pub struct StatementCache<T: StatementTable> {
    map: InnerMap<T>,
}

impl<T: StatementTable> StatementCache<T> {
    // TEMP:
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { map: Default::default() }
    }

    pub fn get_or_insert(&self, id: T::Id, (crate_db, crate_record): &CrateValue) -> Result<&T, RedefmtDecoderError>
    where
        T::Id: Copy + AsRef<u16> + Hash + Eq,
    {
        let statement = match self.map.get(&id) {
            Some(statement) => statement,
            None => {
                let Some(statement) = crate_db.find_by_id(id)? else {
                    return Err(RedefmtDecoderError::UnknownStatement(
                        *id.as_ref(),
                        T::NAME,
                        crate_record.name().clone(),
                    ));
                };

                self.map.insert(id, Box::new(statement))
            }
        };

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
