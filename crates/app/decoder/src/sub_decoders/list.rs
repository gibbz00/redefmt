use bytes::BytesMut;
use redefmt_core::frame::{PointerWidth, TypeHint};

use crate::*;

pub struct ListValueDecoder<'cache> {
    pointer_width: PointerWidth,
    expected_length: usize,
    buffer: Vec<Value<'cache>>,
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
        stores: &Stores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<Vec<Value<'cache>>>, RedefmtDecoderError> {
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

        Ok(Some(values))
    }

    pub fn decode_dyn_list(
        &mut self,
        stores: &Stores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<Vec<Value<'cache>>>, RedefmtDecoderError> {
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

        Ok(Some(values))
    }

    fn get_or_insert_element_type_hint(&mut self, src: &mut BytesMut) -> Result<Option<TypeHint>, RedefmtDecoderError> {
        if let Some(type_hint) = self.element_type_hint {
            return Ok(Some(type_hint));
        }

        let maybe_type_hint = DecoderUtils::get_type_hint(src)?;
        self.element_type_hint = maybe_type_hint;
        Ok(maybe_type_hint)
    }
}
