use bytes::{Buf, BytesMut};
use redefmt_core::{frame::PointerWidth, identifiers::TypeStructureId};
use redefmt_db::statement_table::type_structure::{StructVariant, TypeStructure, TypeStructureVariant};

use crate::*;

enum TypeStructureValueSubDecoder<'cache> {
    Struct(StructDecoder<'cache>),
    Enum(EnumDecoder<'cache>),
}

enum TypeStructureDecoderWants<'cache> {
    SubDecoder,
    Value(&'cache TypeStructure<'static>, TypeStructureValueSubDecoder<'cache>),
}

pub struct TypeStructureDecoder<'cache> {
    pointer_width: PointerWidth,
    crate_context: CrateContext<'cache>,
    decoder_stage: TypeStructureDecoderWants<'cache>,
}

impl<'cache> TypeStructureDecoder<'cache> {
    pub fn new(pointer_width: PointerWidth, crate_context: CrateContext<'cache>) -> Self {
        Self {
            pointer_width,
            crate_context,
            decoder_stage: TypeStructureDecoderWants::SubDecoder,
        }
    }

    pub fn decode(
        &mut self,
        stores: &Stores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<TypeStructureValue<'cache>>, RedefmtDecoderError> {
        match &mut self.decoder_stage {
            TypeStructureDecoderWants::SubDecoder => {
                let Ok(type_structure_id) = src.try_get_u16().map(TypeStructureId::new) else {
                    return Ok(None);
                };

                let type_structure = stores
                    .cache
                    .type_structure
                    .get_or_insert(type_structure_id, self.crate_context)?;

                let sub_decoder = match &type_structure.variant {
                    TypeStructureVariant::Struct(struct_variant) => match struct_variant {
                        StructVariant::Unit => {
                            return Ok(Some(TypeStructureValue {
                                name: &type_structure.name,
                                variant: TypeStructureVariantValue::Struct(StructVariantValue::Unit),
                            }));
                        }
                        StructVariant::Tuple(tuple_length) => TypeStructureValueSubDecoder::Struct(
                            StructDecoder::tuple(self.pointer_width, *tuple_length),
                        ),
                        StructVariant::Named(field_names) => {
                            TypeStructureValueSubDecoder::Struct(StructDecoder::named(self.pointer_width, field_names))
                        }
                    },
                    TypeStructureVariant::Enum(variants) => {
                        TypeStructureValueSubDecoder::Enum(EnumDecoder::WantsIndex { variants })
                    }
                };

                self.decoder_stage = TypeStructureDecoderWants::Value(type_structure, sub_decoder);

                self.decode(stores, src)
            }
            TypeStructureDecoderWants::Value(type_structure, sub_decoder) => {
                let decoded_variant = match sub_decoder {
                    TypeStructureValueSubDecoder::Struct(struct_decoder) => struct_decoder
                        .decode(stores, src)?
                        .map(TypeStructureVariantValue::Struct),
                    TypeStructureValueSubDecoder::Enum(enum_decoder) => enum_decoder
                        .decode(self.pointer_width, stores, src)?
                        .map(TypeStructureVariantValue::Enum),
                };

                let maybe_value =
                    decoded_variant.map(|variant| TypeStructureValue { name: &type_structure.name, variant });

                Ok(maybe_value)
            }
        }
    }
}
