use std::collections::{HashMap, hash_map::Entry};

use redefmt_common::identifiers::PrintStatementId;
use redefmt_db::{Table, statement_table::print::PrintStatement};

use crate::*;

type InnerMap = HashMap<PrintStatementId, PrintStatement<'static>>;

#[derive(Default)]
pub struct PrintStatementCache {
    map: InnerMap,
}

impl PrintStatementCache {
    pub fn fetch_and_save(
        &mut self,
        id: PrintStatementId,
        (print_crate_db, print_crate_record): &CrateValue,
    ) -> Result<&PrintStatement<'static>, RedefmtDecoderError> {
        if let Entry::Vacant(entry) = self.map.entry(id) {
            let Some(print_statement) = print_crate_db.find_by_id(id)? else {
                return Err(RedefmtDecoderError::UnknownPrintStatement(
                    id,
                    print_crate_record.name().clone(),
                ));
            };

            entry.insert(print_statement);
        }

        let print_statement = self.map.get(&id).expect("print entry not cached");

        Ok(print_statement)
    }
}

#[cfg(test)]
mod ext {
    use super::*;

    impl PrintStatementCache {
        pub fn inner(&self) -> &InnerMap {
            &self.map
        }
    }
}
