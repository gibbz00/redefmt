use redefmt_core::codec::frame::{PointerWidth, TypeHint};
use redefmt_db::statement_table::stored_format_expression::StoredFormatExpression;
use tokio_util::bytes::BytesMut;

use crate::*;

struct SegmentValueContext<'cache> {
    type_hint: TypeHint,
    value_decoder: ValueDecoder<'cache>,
}

pub struct SegmentsDecoder<'cache> {
    pointer_width: PointerWidth,
    current_value: Option<SegmentValueContext<'cache>>,
    pub(crate) stored_expression: &'cache StoredFormatExpression<'static>,
    pub(crate) decoded_values: DecodedValues<'cache>,
}

impl<'cache> SegmentsDecoder<'cache> {
    pub fn new(pointer_width: PointerWidth, stored_expression: &'cache StoredFormatExpression<'static>) -> Self {
        let decoded_values = DecodedValues::new_with_capacity(stored_expression);
        Self { stored_expression, pointer_width, current_value: None, decoded_values }
    }

    pub fn decode(&mut self, stores: &Stores<'cache>, src: &mut BytesMut) -> Result<Option<()>, RedefmtDecoderError> {
        if let Some(current_value_context) = self.current_value.take() {
            let SegmentValueContext { type_hint, mut value_decoder } = current_value_context;

            match value_decoder.decode(stores, src)? {
                Some(value) => {
                    self.decoded_values.push(value, self.stored_expression);
                    self.current_value = None;
                }
                None => {
                    self.current_value = Some(SegmentValueContext { type_hint, value_decoder });
                    return Ok(None);
                }
            }
        }

        while !self.decoded_values.is_filled(self.stored_expression) {
            let Some(type_hint) = DecoderUtils::get_type_hint(src)? else {
                return Ok(None);
            };

            let mut value_decoder = ValueDecoder::new(self.pointer_width, type_hint);

            match value_decoder.decode(stores, src)? {
                Some(value) => {
                    self.decoded_values.push(value, self.stored_expression);
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
