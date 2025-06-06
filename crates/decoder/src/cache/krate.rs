use std::path::Path;

use elsa::sync::FrozenMap;
use redefmt_core::identifiers::CrateId;
use redefmt_db::{CrateDb, DbClient, MainDb, Table, crate_table::Crate};

use crate::*;

type CrateValue = (DbClient<CrateDb>, Crate<'static>);

#[derive(Clone, Copy)]
pub struct CrateContext<'cache> {
    pub id: CrateId,
    pub record: &'cache Crate<'static>,
    pub db: &'cache DbClient<CrateDb>,
}

#[derive(Default)]
pub struct CrateCache {
    map: FrozenMap<CrateId, Box<CrateValue>>,
}

impl CrateCache {
    pub fn get_or_insert(
        &self,
        id: CrateId,
        db: &DbClient<MainDb>,
        state_dir: &Path,
    ) -> Result<CrateContext<'_>, RedefmtDecoderError> {
        let (db, record) = match self.map.get(&id) {
            Some(value) => value,
            None => {
                let Some(crate_record) = db.find_by_id(id)? else {
                    return Err(RedefmtDecoderError::UnknownCrate(id));
                };

                let crate_db = DbClient::new_crate(state_dir, &crate_record.name)?;

                self.map.insert(id, Box::new((crate_db, crate_record)))
            }
        };

        Ok(CrateContext { id, record, db })
    }
}

#[cfg(test)]
mod ext {
    use super::*;

    impl CrateCache {
        pub fn inner(&self) -> &FrozenMap<CrateId, Box<CrateValue>> {
            &self.map
        }
    }
}
