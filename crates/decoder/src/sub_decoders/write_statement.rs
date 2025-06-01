use redefmt_core::{codec::frame::PointerWidth, identifiers::WriteStatementId};
use redefmt_db::statement_table::write::WriteStatement;
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

pub struct WriteStatementDecoder<'cache> {
    pointer_width: PointerWidth,
    write_crate: CrateContext<'cache>,
    stage: WriteStatementDecoderStage<'cache>,
}

#[derive(Default)]
enum WriteStatementDecoderStage<'cache> {
    #[default]
    New,
    Segments(Box<SegmentsDecoder<'cache>>),
    TypeStructure(TypeStructureDecoder<'cache>),
}

impl<'cache> WriteStatementDecoder<'cache> {
    pub fn new(pointer_width: PointerWidth, write_crate: CrateContext<'cache>) -> Self {
        Self { pointer_width, write_crate, stage: WriteStatementDecoderStage::New }
    }

    pub fn decode(
        &mut self,
        stores: &Stores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<Value<'cache>>, RedefmtDecoderError> {
        let current_stage = std::mem::take(&mut self.stage);

        match current_stage {
            WriteStatementDecoderStage::New => {
                let Ok(write_statement_id) = src.try_get_u16().map(WriteStatementId::new) else {
                    return Ok(None);
                };

                let write_statement = stores
                    .cache
                    .write_statement
                    .get_or_insert(write_statement_id, self.write_crate)?;

                match write_statement {
                    WriteStatement::FormatExpression(format_expression) => {
                        let segment_decoder = SegmentsDecoder::new(self.pointer_width, format_expression);
                        self.stage = WriteStatementDecoderStage::Segments(Box::new(segment_decoder));
                        self.decode(stores, src)
                    }
                    WriteStatement::TypeStructure(type_structure) => {
                        let type_structure_decoder = TypeStructureDecoder::new(self.pointer_width, type_structure);
                        self.stage = WriteStatementDecoderStage::TypeStructure(type_structure_decoder);
                        self.decode(stores, src)
                    }
                }
            }
            WriteStatementDecoderStage::Segments(mut segment_decoder) => {
                if segment_decoder.decode(stores, src)?.is_none() {
                    self.stage = WriteStatementDecoderStage::Segments(segment_decoder);
                    return Ok(None);
                }

                // flatten stored expression to avoid leaking db crate in public API
                Ok(Some(Value::NestedFormatExpression {
                    expression: &segment_decoder.stored_expression.expression,
                    append_newline: segment_decoder.stored_expression.append_newline,
                    decoded_values: segment_decoder.decoded_args,
                }))
            }
            WriteStatementDecoderStage::TypeStructure(mut type_structure_decoder) => {
                let Some(type_structure_value) = type_structure_decoder.decode(stores, src)? else {
                    self.stage = WriteStatementDecoderStage::TypeStructure(type_structure_decoder);
                    return Ok(None);
                };

                Ok(Some(Value::Type(type_structure_value)))
            }
        }
    }
}
