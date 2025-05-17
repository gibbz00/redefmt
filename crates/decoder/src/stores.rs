use std::path::PathBuf;

use redefmt_common::identifiers::CrateId;
use redefmt_db::{DbClient, MainDb, StateDir, statement_table::print::PrintStatement};

use crate::*;

pub struct DecoderStores<'caches> {
    state_dir: PathBuf,
    pub(crate) main_db: DbClient<MainDb>,
    pub(crate) crate_cache: &'caches CrateCache,
    pub(crate) print_statement_cache: &'caches StatementCache<PrintStatement<'static>>,
}

impl<'caches> DecoderStores<'caches> {
    pub fn new(
        crate_cache: &'caches CrateCache,
        print_statement_cache: &'caches StatementCache<PrintStatement<'static>>,
    ) -> Result<Self, RedefmtDecoderError> {
        let dir = StateDir::resolve()?;
        Self::new_impl(crate_cache, print_statement_cache, dir)
    }

    pub fn new_impl(
        crate_cache: &'caches CrateCache,
        print_statement_cache: &'caches StatementCache<PrintStatement<'static>>,
        state_dir: PathBuf,
    ) -> Result<Self, RedefmtDecoderError> {
        let main_db = DbClient::new_main(&state_dir)?;

        Ok(Self { state_dir, main_db, crate_cache, print_statement_cache })
    }

    pub fn get_or_insert_crate(&self, crate_id: CrateId) -> Result<&'caches CrateValue, RedefmtDecoderError> {
        self.crate_cache.get_or_insert(crate_id, &self.main_db, &self.state_dir)
    }
}
