use redefmt_db::statement_table::{print::PrintStatement, type_structure::TypeStructure, write::WriteStatement};

use crate::*;

#[derive(Default)]
pub struct RedefmtDecoderCache {
    pub(crate) krate: CrateCache,
    pub(crate) print_statement: StatementCache<PrintStatement<'static>>,
    pub(crate) write_statement: StatementCache<WriteStatement<'static>>,
    pub(crate) type_structure: StatementCache<TypeStructure<'static>>,
}
