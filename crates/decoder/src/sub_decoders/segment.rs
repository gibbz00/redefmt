use redefmt_core::codec::frame::{PointerWidth, TypeHint};
use redefmt_db::statement_table::stored_format_expression::StoredFormatExpression;
use tokio_util::bytes::BytesMut;

use crate::*;

struct SegmentValueContext<'cache> {
    type_hint: TypeHint,
    value_decoder: ValueDecoder<'cache>,
}

pub struct SegmentsDecoder<'cache> {
    expected_arg_count: usize,
    pointer_width: PointerWidth,
    current_value: Option<SegmentValueContext<'cache>>,
    pub(crate) stored_expression: &'cache StoredFormatExpression<'static>,
    pub(crate) decoded_args: Vec<ComplexValue<'cache>>,
}

impl<'cache> SegmentsDecoder<'cache> {
    pub fn new(pointer_width: PointerWidth, stored_expression: &'cache StoredFormatExpression<'static>) -> Self {
        let expected_arg_count = stored_expression.expression().provided_args().count();
        let decoded_args = Vec::with_capacity(expected_arg_count);

        Self {
            stored_expression,
            expected_arg_count,
            pointer_width,
            current_value: None,
            decoded_args,
        }
    }

    pub fn decode(&mut self, stores: &Stores<'cache>, src: &mut BytesMut) -> Result<Option<()>, RedefmtDecoderError> {
        if let Some(current_value_context) = self.current_value.take() {
            let SegmentValueContext { type_hint, mut value_decoder } = current_value_context;

            match value_decoder.decode(stores, src)? {
                Some(value) => {
                    self.decoded_args.push(value);
                    self.current_value = None;
                }
                None => {
                    self.current_value = Some(SegmentValueContext { type_hint, value_decoder });
                    return Ok(None);
                }
            }
        }

        while self.decoded_args.len() < self.expected_arg_count {
            let Some(type_hint) = DecoderUtils::get_type_hint(src)? else {
                return Ok(None);
            };

            let mut value_decoder = ValueDecoder::new(self.pointer_width, type_hint);

            match value_decoder.decode(stores, src)? {
                Some(value) => {
                    self.decoded_args.push(value);
                }
                None => {
                    self.current_value = Some(SegmentValueContext { type_hint, value_decoder });
                    return Ok(None);
                }
            }
        }

        Ok(Some(()))
    }
}
