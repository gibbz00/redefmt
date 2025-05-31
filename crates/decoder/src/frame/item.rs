use redefmt_args::deferred::DeferredFormatExpression;
use redefmt_core::codec::frame::{Level, Stamp};
use redefmt_db::statement_table::print::PrintStatement;

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
    pub level: Option<Level>,
    pub stamp: Option<Stamp>,
    pub file_name: &'cache str,
    pub file_line: u32,
    pub format_expression: &'cache DeferredFormatExpression<'static>,
    pub append_newline: bool,
    pub decoded_values: Vec<Value<'cache>>,
}

impl<'cache> RedefmtFrame<'cache> {
    // flattens `PrintStatement` to avoid exposing it in the public API.
    pub(crate) fn new(
        level: Option<Level>,
        stamp: Option<Stamp>,
        print_stratement: &'cache PrintStatement<'static>,
        decoded_values: Vec<Value<'cache>>,
    ) -> Self {
        Self {
            level,
            stamp,
            file_name: print_stratement.location().file().as_ref(),
            file_line: *print_stratement.location().line(),
            format_expression: print_stratement.format_expression().expression(),
            append_newline: *print_stratement.format_expression().append_newline(),
            decoded_values,
        }
    }
}
