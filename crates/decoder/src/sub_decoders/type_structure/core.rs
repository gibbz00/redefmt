use redefmt_core::codec::frame::PointerWidth;
use redefmt_db::statement_table::write::{StructVariant, TypeStructure, TypeStructureVariant};
use tokio_util::bytes::BytesMut;

use crate::*;

enum TypeStructureValueSubDecoder<'cache> {
    Struct(StructDecoder<'cache>),
    Enum(EnumDecoder<'cache>),
}

pub struct TypeStructureDecoder<'cache> {
    pointer_width: PointerWidth,
    type_structure: &'cache TypeStructure,
    sub_decoder: Option<TypeStructureValueSubDecoder<'cache>>,
}

impl<'cache> TypeStructureDecoder<'cache> {
    pub fn new(pointer_width: PointerWidth, type_structure: &'cache TypeStructure) -> Self {
        Self { pointer_width, type_structure, sub_decoder: None }
    }

    pub fn decode(
        &mut self,
        stores: &Stores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<TypeStructureValue<'cache>>, RedefmtDecoderError> {
        let mut sub_decoder = match self.sub_decoder.take() {
            Some(sub_decoder) => sub_decoder,
            None => match &self.type_structure.variant {
                TypeStructureVariant::Struct(struct_variant) => match struct_variant {
                    StructVariant::Unit => {
                        return Ok(Some(TypeStructureValue {
                            name: &self.type_structure.name,
                            variant: TypeStructureVariantValue::Struct(StructVariantValue::Unit),
                        }));
                    }
                    StructVariant::Tuple(tuple_length) => {
                        TypeStructureValueSubDecoder::Struct(StructDecoder::tuple(self.pointer_width, *tuple_length))
                    }
                    StructVariant::Named(field_names) => {
                        TypeStructureValueSubDecoder::Struct(StructDecoder::named(self.pointer_width, field_names))
                    }
                },
                TypeStructureVariant::Enum(variants) => {
                    TypeStructureValueSubDecoder::Enum(EnumDecoder::WantsIndex { variants })
                }
            },
        };

        let decoded_value = match &mut sub_decoder {
            TypeStructureValueSubDecoder::Struct(struct_decoder) => struct_decoder
                .decode(stores, src)?
                .map(TypeStructureVariantValue::Struct),
            TypeStructureValueSubDecoder::Enum(enum_decoder) => enum_decoder
                .decode(self.pointer_width, stores, src)?
                .map(TypeStructureVariantValue::Enum),
        };

        match decoded_value {
            Some(decoded_variant) => {
                let type_structure_value =
                    TypeStructureValue { name: &self.type_structure.name, variant: decoded_variant };
                Ok(Some(type_structure_value))
            }
            None => {
                // important to put it back for next time as it was retrieved with `take()`
                self.sub_decoder = Some(sub_decoder);
                Ok(None)
            }
        }
    }
}
