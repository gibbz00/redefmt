use redefmt_db::{
    CrateDb, DbClient, StateDir, Table,
    crate_table::{Crate, CrateName, CrateTable},
};
use redefmt_internal::identifiers::CrateId;

use crate::*;

pub struct DbClients {
    pub crate_db: DbClient<CrateDb>,
    pub crate_id: CrateId,
}

impl DbClients {
    pub fn new() -> Result<Self, RedefmtMacroError> {
        let state_dir = StateDir::resolve()?;

        let main_db = DbClient::new_main(&state_dir)?;

        let crate_name = Self::crate_name()?;

        let crate_db = DbClient::new_crate(&state_dir, &crate_name)?;

        let crate_id = match main_db.find_crate_by_name(&crate_name)? {
            Some((id, _)) => id,
            None => {
                let record = Crate::new(crate_name);
                main_db.insert(&record)?
            }
        };

        Ok(Self { crate_db, crate_id })
    }

    fn crate_name() -> Result<CrateName<'static>, RedefmtMacroError> {
        let name_str = std::env::var("CARGO_PKG_NAME")?;
        let crate_name = CrateName::new(name_str)?;
        Ok(crate_name)
    }
}
