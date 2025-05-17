use std::{iter::Peekable, slice::Iter as SliceIter};

use redefmt_args::FormatOptions;
use redefmt_common::codec::frame::{PointerWidth, TypeHint};
use redefmt_db::statement_table::Segment;
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

struct SegmentValueContext<'caches> {
    type_hint: TypeHint,
    format_options: &'caches FormatOptions<'static>,
    value_decoder: ValueDecoder<'caches>,
}

pub struct SegmentsDecoder<'caches> {
    pointer_width: PointerWidth,
    current_value: Option<SegmentValueContext<'caches>>,
    pub(crate) segments: Peekable<SliceIter<'caches, Segment<'static>>>,
    pub(crate) decoded_segments: Vec<DecodedSegment<'caches>>,
}

impl<'caches> SegmentsDecoder<'caches> {
    pub fn new(pointer_width: PointerWidth, segments: &'caches [Segment<'static>]) -> Self {
        let segments_iter = segments.iter().peekable();

        // IMPROVEMENT: count args in segments?
        let decoded_segments = Vec::new();

        Self {
            pointer_width,
            segments: segments_iter,
            current_value: None,
            decoded_segments,
        }
    }

    pub fn decode(
        &mut self,
        stores: &DecoderStores<'caches>,
        src: &mut BytesMut,
    ) -> Result<Option<()>, RedefmtDecoderError> {
        if let Some(current_value_context) = self.current_value.take() {
            let SegmentValueContext { type_hint, format_options, mut value_decoder } = current_value_context;

            match value_decoder.decode(stores, src)? {
                Some(value) => {
                    self.decoded_segments.push(DecodedSegment::Value(value, format_options));
                    self.current_value = None;
                }
                None => {
                    self.current_value = Some(SegmentValueContext { type_hint, format_options, value_decoder });
                    return Ok(None);
                }
            }
        }

        while let Some(next_segment) = self.segments.peek() {
            match next_segment {
                Segment::Str(str) => {
                    self.decoded_segments.push(DecodedSegment::Str(str));
                    self.segments.next();
                }
                Segment::Arg(format_options) => {
                    let Ok(type_hint_repr) = src.try_get_u8() else {
                        return Ok(None);
                    };

                    let type_hint = TypeHint::from_repr(type_hint_repr)
                        .ok_or(RedefmtDecoderError::UnknownTypeHint(type_hint_repr))?;

                    let mut value_decoder = ValueDecoder::new(self.pointer_width, type_hint);

                    match value_decoder.decode(stores, src)? {
                        Some(value) => {
                            self.decoded_segments.push(DecodedSegment::Value(value, format_options));
                            self.segments.next();
                        }
                        None => {
                            self.current_value = Some(SegmentValueContext { type_hint, format_options, value_decoder });
                            return Ok(None);
                        }
                    }
                }
            }
        }

        Ok(Some(()))
    }
}
