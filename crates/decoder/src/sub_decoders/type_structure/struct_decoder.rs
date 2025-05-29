use redefmt_core::codec::frame::PointerWidth;
use tokio_util::bytes::BytesMut;

use crate::*;

pub enum StructDecoder<'cache> {
    TupleStruct {
        list_decoder: ListValueDecoder<'cache>,
    },
    NamedStruct {
        field_names: &'cache Vec<String>,
        list_decoder: ListValueDecoder<'cache>,
    },
}

impl<'cache> StructDecoder<'cache> {
    pub fn tuple(pointer_width: PointerWidth, tuple_length: u8) -> Self {
        let tuple_length = tuple_length as usize;
        let list_decoder = ListValueDecoder::new(pointer_width, tuple_length);
        Self::TupleStruct { list_decoder }
    }

    pub fn named(pointer_width: PointerWidth, field_names: &'cache Vec<String>) -> Self {
        let list_decoder = ListValueDecoder::new(pointer_width, field_names.len());
        Self::NamedStruct { field_names, list_decoder }
    }
}

impl<'cache> StructDecoder<'cache> {
    pub fn decode(
        &mut self,
        stores: &Stores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<StructVariantValue<'cache>>, RedefmtDecoderError> {
        let struct_variant_value = match self {
            StructDecoder::TupleStruct { list_decoder } => {
                let Some(decoded_fields) = list_decoder.decode_dyn_list(stores, src)? else {
                    return Ok(None);
                };

                StructVariantValue::Tuple(decoded_fields)
            }
            StructDecoder::NamedStruct { field_names, list_decoder } => {
                let Some(decoded_fields) = list_decoder.decode_dyn_list(stores, src)? else {
                    return Ok(None);
                };

                let decoded_fields = field_names.iter().zip(decoded_fields).collect();

                StructVariantValue::Named(decoded_fields)
            }
        };

        Ok(Some(struct_variant_value))
    }
}
