use std::{iter::Peekable, slice::Iter as SliceIter};

use redefmt_args::FormatOptions;
use redefmt_common::codec::frame::TypeHint;
use redefmt_db::statement_table::Segment;

use crate::ValueContext;

pub struct SegmentContext<'caches> {
    pub segments: Peekable<SliceIter<'caches, Segment<'static>>>,
    pub current_value: Option<SegmentValueContext<'caches>>,
}

impl<'caches> SegmentContext<'caches> {
    pub fn new(segments: &'caches [Segment<'static>]) -> Self {
        let segments_iter = segments.iter().peekable();
        Self { segments: segments_iter, current_value: None }
    }
}

pub struct SegmentValueContext<'caches> {
    pub type_hint: TypeHint,
    pub format_options: &'caches FormatOptions<'static>,
    pub value_context: ValueContext,
}
