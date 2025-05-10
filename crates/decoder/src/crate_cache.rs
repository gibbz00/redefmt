use std::{
    collections::{HashMap, hash_map::Entry},
    path::Path,
};

use redefmt_common::identifiers::CrateId;
use redefmt_db::{CrateDb, DbClient, MainDb, Table, crate_table::Crate};

use crate::RedefmtDecoderError;

pub type CrateValue = (DbClient<CrateDb>, Crate<'static>);

#[derive(Default)]
pub struct CrateCache {
    map: HashMap<CrateId, CrateValue>,
}

impl CrateCache {
    pub fn fetch_and_save(
        &mut self,
        id: CrateId,
        db: &DbClient<MainDb>,
        state_dir: &Path,
    ) -> Result<&CrateValue, RedefmtDecoderError> {
        if let Entry::Vacant(vacant_entry) = self.map.entry(id) {
            let Some(crate_record) = db.find_by_id(id)? else {
                return Err(RedefmtDecoderError::UnknownCrate(id));
            };

            let crate_db = DbClient::new_crate(state_dir, crate_record.name())?;

            vacant_entry.insert((crate_db, crate_record));
        }

        let crate_value = self.map.get(&id).expect("crate value not stored in cache");

        Ok(crate_value)
    }

    pub fn get(&self, id: CrateId) -> Option<&CrateValue> {
        self.map.get(&id)
    }
}

#[cfg(test)]
mod ext {
    use super::*;

    impl CrateCache {
        pub fn inner(&self) -> &HashMap<CrateId, CrateValue> {
            &self.map
        }
    }
}
