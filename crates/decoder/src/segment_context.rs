use redefmt_args::FormatOptions;
use redefmt_common::codec::frame::TypeHint;
use redefmt_db::statement_table::Segment;

use crate::ValueContext;

pub struct SegmentContext {
    /// In reverse order to make taking next cheap
    pub segments: Vec<Segment<'static>>,
    pub current_value: Option<SegmentValueContext>,
}

impl SegmentContext {
    pub fn new(segments: Vec<Segment<'static>>) -> Self {
        let reversed = segments.into_iter().rev().collect();
        Self { segments: reversed, current_value: None }
    }
}

pub struct SegmentValueContext {
    pub type_hint: TypeHint,
    pub format_options: FormatOptions<'static>,
    pub value_context: ValueContext,
}
