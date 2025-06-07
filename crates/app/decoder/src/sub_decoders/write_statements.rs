use bytes::{Buf, BytesMut};
use redefmt_core::{
    frame::PointerWidth,
    identifiers::{CrateId, WriteStatementId},
    write::StatementWriterHint,
};

use crate::*;

pub enum WriteStatementsDecoderWants<'cache> {
    WriterHint,
    StatementsCrate,
    Segments(CrateContext<'cache>),
    Value(Box<SegmentsDecoder<'cache>>),
}

pub struct WriteStatementsDecoder<'cache> {
    pointer_width: PointerWidth,
    stage: WriteStatementsDecoderWants<'cache>,
    decoded_statements: Vec<WriteStatementValue<'cache>>,
}

impl<'cache> WriteStatementsDecoder<'cache> {
    pub fn new(pointer_width: PointerWidth) -> Self {
        Self {
            pointer_width,
            stage: WriteStatementsDecoderWants::WriterHint,
            decoded_statements: Default::default(),
        }
    }

    pub fn decode(
        &mut self,
        stores: &Stores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<Vec<WriteStatementValue<'cache>>>, RedefmtDecoderError> {
        let stage = std::mem::replace(&mut self.stage, WriteStatementsDecoderWants::WriterHint);

        match stage {
            WriteStatementsDecoderWants::WriterHint => {
                let Some(writer_hint) = DecoderUtils::get_statement_writer_hint(src)? else {
                    return Ok(None);
                };

                match writer_hint {
                    StatementWriterHint::End => {
                        let decoded_statements = std::mem::take(&mut self.decoded_statements);
                        Ok(Some(decoded_statements))
                    }
                    StatementWriterHint::Continue => {
                        self.stage = WriteStatementsDecoderWants::StatementsCrate;
                        self.decode(stores, src)
                    }
                }
            }
            WriteStatementsDecoderWants::StatementsCrate => {
                // IMPROVEMENT: This will always return the same Crate ID.
                //
                // It should ideally only need to be written once when starting
                // the statement writer, rather than once for each statement,
                // (TH = TypeHint SH = StatementWriterHint):
                //
                // Current bytes: TH::WriteStatements + CrateId +  (SH::Continue
                // + WriteStatement ID + Values)* + SH::End
                //
                // Improved bytes: TH::WriteStatements +  (SH::Continue + Crate
                // Id + WriteStatement ID + Values)* + SH::End
                //
                // API would become need to become a bit janky, since the writer
                // would need to be returned from a proc macro which resolves
                // the crate ID, and not from the `Formatter::statements_writer`
                // method.
                let Ok(crate_id) = src.try_get_u16().map(CrateId::new) else {
                    self.stage = WriteStatementsDecoderWants::StatementsCrate;
                    return Ok(None);
                };

                let statements_crate = stores.get_or_insert_crate(crate_id)?;

                self.stage = WriteStatementsDecoderWants::Segments(statements_crate);

                self.decode(stores, src)
            }
            WriteStatementsDecoderWants::Segments(crate_context) => {
                let Ok(write_statement_id) = src.try_get_u16().map(WriteStatementId::new) else {
                    self.stage = WriteStatementsDecoderWants::Segments(crate_context);
                    return Ok(None);
                };

                let write_statement = stores
                    .cache
                    .write_statement
                    .get_or_insert(write_statement_id, crate_context)?;

                let segment_decoder = SegmentsDecoder::new(self.pointer_width, &write_statement.0);

                self.stage = WriteStatementsDecoderWants::Value(Box::new(segment_decoder));

                self.decode(stores, src)
            }
            WriteStatementsDecoderWants::Value(mut segments_decoder) => {
                if segments_decoder.decode(stores, src)?.is_none() {
                    self.stage = WriteStatementsDecoderWants::Value(segments_decoder);
                    return Ok(None);
                }

                // flattens stored expression to avoid leaking DB crate type in public API
                let decoded_statement = WriteStatementValue {
                    expression: &segments_decoder.stored_expression.format_string,
                    append_newline: segments_decoder.stored_expression.append_newline,
                    decoded_values: segments_decoder.decoded_values,
                };

                self.decoded_statements.push(decoded_statement);

                // IMPROVEMENT: remove recursion to avoid theoretical stack overflows?
                self.decode(stores, src)
            }
        }
    }
}
