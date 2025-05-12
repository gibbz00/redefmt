use redefmt_args::FormatOptions;
use redefmt_common::codec::frame::Stamp;
use redefmt_db::statement_table::print::PrintInfo;

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
///
/// Of the primitives, only the string needs to send a length hint.
/// Of the collections, all send a length hint, and a type new hint.
/// The map collection sends an extra type hint.
/// Write_id will in turn tell if if is a write_statement or type_structure.
#[derive(Debug, PartialEq)]
pub struct RedefmtFrame {
    pub(crate) stamp: Option<Stamp>,
    // IMPROVEMENT: use Arc
    pub(crate) print_info: PrintInfo<'static>,
    pub(crate) segments: Vec<DecodedSegment>,
}

#[derive(Debug, PartialEq)]
pub enum DecodedSegment {
    // IMPROVEMENT: use Arc<str>
    Str(String),
    // IMPROVEMENT: use Arc for FormatOptions
    Value(Value, FormatOptions<'static>),
}
