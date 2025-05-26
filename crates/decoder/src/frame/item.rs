use redefmt_db::statement_table::print::PrintStatement;
use redefmt_core::codec::frame::Stamp;

use crate::*;

/// # Codec structure:
///
/// ```txt
/// frame := <header>[<stamp>]<print_id>(content)*
/// content := <type_hint>[<length_hint>](<type_hint>{1,2})[<type_bytes>]
/// print_id := <crate_id><print_statement_id>
/// write_id := <crate_id><write_statement_id>
/// type_hint (u8) := primitives (13) | collection (4) | write_id (1)
/// ```
#[derive(Debug, PartialEq)]
pub struct RedefmtFrame<'cache> {
    pub(crate) stamp: Option<Stamp>,
    pub(crate) print_statement: &'cache PrintStatement<'static>,
    pub(crate) decoded_values: Vec<ComplexValue<'cache>>,
}
