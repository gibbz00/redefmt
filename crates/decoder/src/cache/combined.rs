use redefmt_db::statement_table::print::PrintStatement;

use crate::*;

#[derive(Default)]
pub struct DecoderCache {
    pub(crate) krate: CrateCache,
    pub(crate) print_statement: StatementCache<PrintStatement<'static>>,
}
