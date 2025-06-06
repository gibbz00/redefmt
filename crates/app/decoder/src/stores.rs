use std::path::PathBuf;

use redefmt_core::identifiers::CrateId;
use redefmt_db::{DbClient, MainDb};

use crate::*;

pub struct Stores<'cache> {
    state_dir: PathBuf,
    pub(crate) main_db: DbClient<MainDb>,
    pub(crate) cache: &'cache RedefmtDecoderCache,
}

impl<'cache> Stores<'cache> {
    pub fn new(cache: &'cache RedefmtDecoderCache, state_dir: PathBuf) -> Result<Self, RedefmtDecoderError> {
        let main_db = DbClient::new_main(&state_dir)?;

        Ok(Self { state_dir, main_db, cache })
    }

    pub fn get_or_insert_crate(&self, crate_id: CrateId) -> Result<CrateContext<'cache>, RedefmtDecoderError> {
        self.cache.krate.get_or_insert(crate_id, &self.main_db, &self.state_dir)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    impl<'cache> Stores<'cache> {
        pub fn mock(cache: &'cache RedefmtDecoderCache) -> (TempDir, Self) {
            let temp_dir = tempfile::tempdir().unwrap();

            let state_dir = temp_dir.path().to_path_buf();

            let stores = Stores::new(cache, state_dir).unwrap();

            (temp_dir, stores)
        }
    }
}
