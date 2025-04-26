use redefmt_args::FormatOptions;

use crate::*;

pub enum DisplaySegment {
    // TODO: just store string directly?
    Str(StringDocumentId),
    Arg(Value, FormatOptions<'static>),
}
