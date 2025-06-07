use bytes::BytesMut;
use redefmt_core::frame::PointerWidth;
use redefmt_db::statement_table::type_structure::StructVariant;

use crate::*;

pub enum EnumDecoder<'cache> {
    WantsIndex {
        variants: &'cache Vec<(String, StructVariant)>,
    },
    WantsStructValue {
        variant_name: &'cache String,
        struct_decoder: StructDecoder<'cache>,
    },
}

impl<'cache> EnumDecoder<'cache> {
    pub fn decode(
        &mut self,
        pointer_width: PointerWidth,
        stores: &Stores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<(&'cache str, StructVariantValue<'cache>)>, RedefmtDecoderError> {
        match self {
            EnumDecoder::WantsIndex { variants } => {
                let Some(target_length) = DecoderUtils::get_target_usize(src, pointer_width) else {
                    return Ok(None);
                };

                let variant_index: usize = target_length
                    .try_into()
                    .map_err(|_| RedefmtDecoderError::VariantIndexOverflow(target_length))?;

                let (variant_name, variant_struct_variant) = variants
                    .get(variant_index)
                    .ok_or_else(|| RedefmtDecoderError::UnknownVariantIndex(variant_index))?;

                let struct_decoder = match variant_struct_variant {
                    StructVariant::Unit => {
                        return Ok(Some((variant_name, StructVariantValue::Unit)));
                    }
                    StructVariant::Tuple(tuple_length) => StructDecoder::tuple(pointer_width, *tuple_length),
                    StructVariant::Named(field_names) => StructDecoder::named(pointer_width, field_names),
                };

                EnumDecoder::WantsStructValue { variant_name, struct_decoder }.decode(pointer_width, stores, src)
            }
            EnumDecoder::WantsStructValue { variant_name, struct_decoder } => struct_decoder
                .decode(stores, src)
                .map(|maybe_decoded| maybe_decoded.map(|struct_value| (variant_name.as_str(), struct_value))),
        }
    }
}
