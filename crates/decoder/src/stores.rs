use std::path::PathBuf;

use redefmt_db::{DbClient, MainDb, StateDir};
use redefmt_internal::identifiers::CrateId;

use crate::*;

pub struct DecoderStores<'cache> {
    state_dir: PathBuf,
    pub(crate) main_db: DbClient<MainDb>,
    pub(crate) cache: &'cache DecoderCache,
}

impl<'cache> DecoderStores<'cache> {
    pub fn new(cache: &'cache DecoderCache) -> Result<Self, RedefmtDecoderError> {
        let dir = StateDir::resolve()?;
        Self::new_impl(cache, dir)
    }

    pub fn new_impl(cache: &'cache DecoderCache, state_dir: PathBuf) -> Result<Self, RedefmtDecoderError> {
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

    impl<'cache> DecoderStores<'cache> {
        pub fn mock(cache: &'cache DecoderCache) -> (TempDir, Self) {
            let temp_dir = tempfile::tempdir().unwrap();

            let state_dir = temp_dir.path().to_path_buf();

            let stores = DecoderStores::new_impl(cache, state_dir).unwrap();

            (temp_dir, stores)
        }
    }
}
