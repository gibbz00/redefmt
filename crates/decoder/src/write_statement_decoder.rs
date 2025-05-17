use redefmt_common::{codec::frame::PointerWidth, identifiers::WriteStatementId};
use redefmt_db::statement_table::write::WriteStatement;
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

pub struct WriteStatementDecoder<'caches> {
    pointer_width: PointerWidth,
    write_crate: CrateContext<'caches>,
    stage: WriteStatementDecoderStage<'caches>,
}

#[derive(Default)]
enum WriteStatementDecoderStage<'caches> {
    #[default]
    New,
    Segments(Box<SegmentsDecoder<'caches>>),
}

impl<'caches> WriteStatementDecoder<'caches> {
    pub fn new(pointer_width: PointerWidth, write_crate: CrateContext<'caches>) -> Self {
        Self { pointer_width, write_crate, stage: WriteStatementDecoderStage::New }
    }

    pub fn decode(
        &mut self,
        stores: &DecoderStores<'caches>,
        src: &mut BytesMut,
    ) -> Result<Option<Value>, RedefmtDecoderError> {
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
                    WriteStatement::Segments(segments) => {
                        let segment_decoder = SegmentsDecoder::new(self.pointer_width, segments);
                        self.stage = WriteStatementDecoderStage::Segments(Box::new(segment_decoder));
                        self.decode(stores, src)
                    }
                    WriteStatement::TypeStructure(type_structure) => todo!(),
                }
            }
            WriteStatementDecoderStage::Segments(segments_decoder) => {
                todo!()
            }
        }
    }
}
