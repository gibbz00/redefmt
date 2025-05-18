use redefmt_internal::codec::frame::{PointerWidth, TypeHint};
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

pub struct ListValueDecoder<'cache> {
    pointer_width: PointerWidth,
    expected_length: usize,
    buffer: Vec<ComplexValue<'cache>>,
    element_context: Option<Box<ValueDecoder<'cache>>>,
    element_type_hint: Option<TypeHint>,
}

impl<'cache> ListValueDecoder<'cache> {
    pub fn new(pointer_width: PointerWidth, expected_length: usize) -> Self {
        Self {
            pointer_width,
            expected_length,
            buffer: Vec::with_capacity(expected_length),
            element_context: Default::default(),
            element_type_hint: None,
        }
    }

    pub fn decode_list(
        &mut self,
        stores: &DecoderStores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<Vec<ComplexValue<'cache>>>, RedefmtDecoderError> {
        let Some(element_type_hint) = self.get_or_insert_element_type_hint(src)? else {
            return Ok(None);
        };

        while self.buffer.len() < self.expected_length {
            let element_context = self
                .element_context
                .get_or_insert_with(|| Box::new(ValueDecoder::new(self.pointer_width, element_type_hint)));

            match element_context.decode(stores, src)? {
                Some(value) => {
                    self.buffer.push(value);
                    self.element_context = None;
                }
                None => return Ok(None),
            }
        }

        let values = std::mem::take(&mut self.buffer);

        // No need to clear value_context.list_value_context, decode value should be done here

        Ok(Some(values))
    }

    pub fn decode_dyn_list(
        &mut self,
        stores: &DecoderStores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<Vec<ComplexValue<'cache>>>, RedefmtDecoderError> {
        while self.buffer.len() < self.expected_length {
            let Some(element_type_hint) = self.get_or_insert_element_type_hint(src)? else {
                return Ok(None);
            };

            let element_context = self
                .element_context
                .get_or_insert_with(|| Box::new(ValueDecoder::new(self.pointer_width, element_type_hint)));

            match element_context.decode(stores, src)? {
                Some(value) => {
                    self.buffer.push(value);

                    // Clear element context
                    self.element_type_hint = None;
                    self.element_context = None;
                }
                None => return Ok(None),
            }
        }

        let values = std::mem::take(&mut self.buffer);

        // No need to clear value_context.list_value_context, decode value should be done here

        Ok(Some(values))
    }

    fn get_or_insert_element_type_hint(&mut self, src: &mut BytesMut) -> Result<Option<TypeHint>, RedefmtDecoderError> {
        if let Some(type_hint) = self.element_type_hint {
            return Ok(Some(type_hint));
        }

        let Ok(type_hint_repr) = src.try_get_u8() else {
            return Ok(None);
        };

        let type_hint =
            TypeHint::from_repr(type_hint_repr).ok_or(RedefmtDecoderError::UnknownTypeHint(type_hint_repr))?;

        self.element_type_hint = Some(type_hint);

        Ok(Some(type_hint))
    }
}
