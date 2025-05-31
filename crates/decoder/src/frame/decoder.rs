use redefmt_core::{
    codec::frame::{Header, Stamp},
    identifiers::{CrateId, PrintStatementId},
};
use redefmt_db::StateDir;
use tokio_util::{
    bytes::{Buf, BytesMut},
    codec::Decoder,
};

use crate::*;

pub struct RedefmtDecoder<'cache> {
    // frame indenpendent state
    stores: Stores<'cache>,
    // reset per frame
    stage: FrameDecoderWants<'cache>,
}

impl<'cache> RedefmtDecoder<'cache> {
    pub fn new(cache: &'cache RedefmtDecoderCache) -> Result<Self, RedefmtDecoderError> {
        let state_dir = StateDir::resolve()?;
        let stores = Stores::new(cache, state_dir)?;
        Ok(Self { stores, stage: FrameDecoderWants::Header })
    }
}

impl<'cache> Decoder for RedefmtDecoder<'cache> {
    type Error = RedefmtDecoderError;
    type Item = RedefmtFrame<'cache>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let current_stage = std::mem::take(&mut self.stage);
        match current_stage {
            FrameDecoderWants::Header => {
                let Ok(header_byte) = src.try_get_u8() else {
                    return Ok(None);
                };

                let header = Header::from_bits(header_byte).ok_or(RedefmtDecoderError::UnknownHeader(header_byte))?;

                let next_stage = match header.contains(Header::STAMP) {
                    true => FrameDecoderWants::Stamp(WantsStampStage { header }),
                    false => FrameDecoderWants::PrintCrateId(WantsPrintCrateIdStage { header, stamp: None }),
                };

                self.stage = next_stage;
                self.decode(src)
            }
            FrameDecoderWants::Stamp(stage) => {
                let Ok(stamp) = src.try_get_u64().map(Stamp::new) else {
                    self.stage = FrameDecoderWants::Stamp(stage);
                    return Ok(None);
                };

                self.stage = FrameDecoderWants::PrintCrateId(WantsPrintCrateIdStage {
                    header: stage.header,
                    stamp: Some(stamp),
                });
                self.decode(src)
            }
            FrameDecoderWants::PrintCrateId(stage) => {
                let Ok(print_crate_id) = src.try_get_u16().map(CrateId::new) else {
                    self.stage = FrameDecoderWants::PrintCrateId(stage);
                    return Ok(None);
                };

                let print_crate = self.stores.get_or_insert_crate(print_crate_id)?;

                self.stage = stage.next(print_crate);
                self.decode(src)
            }
            FrameDecoderWants::PrintStatementId(stage) => {
                let Ok(print_statement_id) = src.try_get_u16().map(PrintStatementId::new) else {
                    self.stage = FrameDecoderWants::PrintStatementId(stage);
                    return Ok(None);
                };

                let print_statement = self
                    .stores
                    .cache
                    .print_statement
                    .get_or_insert(print_statement_id, stage.print_crate)?;

                self.stage = stage.next(print_statement);
                self.decode(src)
            }
            FrameDecoderWants::PrintStatement(mut stage) => {
                if stage.segment_decoder.decode(&self.stores, src)?.is_none() {
                    self.stage = FrameDecoderWants::PrintStatement(stage);
                    return Ok(None);
                }

                let item = RedefmtFrame::new(
                    stage.level,
                    stage.stamp,
                    stage.print_statement,
                    stage.segment_decoder.decoded_args,
                );

                self.stage = FrameDecoderWants::Header;

                Ok(Some(item))
            }
        }
    }
}

#[cfg(test)]
mod mock {
    use tempfile::TempDir;

    use super::*;

    impl<'cache> RedefmtDecoder<'cache> {
        pub fn mock(cache: &'cache RedefmtDecoderCache) -> (TempDir, Self) {
            let (temp_dir, stores) = Stores::mock(cache);

            let decoder = RedefmtDecoder { stores, stage: FrameDecoderWants::Header };

            (temp_dir, decoder)
        }
    }
}

#[cfg(test)]
mod tests {
    use redefmt_args::deferred_format_expression;
    use redefmt_core::codec::encoding::{SimpleTestDispatcher, WriteValue};
    use redefmt_db::{
        Table,
        crate_table::{Crate, CrateName},
        statement_table::{
            print::{Location, PrintStatement},
            stored_format_expression::StoredFormatExpression,
        },
    };
    use tokio_util::{bytes::BufMut, codec::Decoder};

    use super::*;

    #[test]
    fn header() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        assert!(matches!(decoder.stage, FrameDecoderWants::Header));

        let expected_header = Header::new(true, None);

        let mut bytes = BytesMut::from_iter([expected_header.bits()]);

        decoder.decode(&mut bytes).unwrap();

        assert!(bytes.is_empty());

        match decoder.stage {
            FrameDecoderWants::Stamp(stage) => {
                assert_eq!(expected_header, stage.header);
            }
            _ => panic!("unexpected stage"),
        }
    }

    #[test]
    fn stamp() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        let header = Header::STAMP;
        let stamp = Stamp::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u8(header.bits());
        bytes.put_u64(*stamp.as_ref());

        decoder.decode(&mut bytes).unwrap();

        assert!(bytes.is_empty());

        match decoder.stage {
            FrameDecoderWants::PrintCrateId(stage) => {
                let actual_stamp = stage.stamp.unwrap();
                assert_eq!(stamp, actual_stamp);
            }
            _ => panic!("unexpected stage"),
        }
    }

    #[test]
    fn print_crate_id() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);

        assert!(cache.krate.inner().is_empty());

        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let expected_crate_value = cache.krate.inner().get(&crate_id).unwrap();

        match decoder.stage {
            FrameDecoderWants::PrintStatementId(stage) => {
                assert_eq!(&expected_crate_value.1, stage.print_crate.record);
            }
            _ => panic!("unexpected stage"),
        }
    }

    #[test]
    fn print_crate_id_not_found_error() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = CrateId::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u16(*crate_id.as_ref());

        let error = decoder.decode(&mut bytes).unwrap_err();
        assert!(matches!(error, RedefmtDecoderError::UnknownCrate(_)));
    }

    #[test]
    fn empty_after_print_crate_id() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn print_statement_id() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let (print_statement_id, print_statement) = seed_print_statement(&decoder, crate_id);

        assert!(!matches!(decoder.stage, FrameDecoderWants::PrintStatement(_)));
        assert!(cache.print_statement.inner().is_empty());

        put_and_decode_print_statement_id(&mut decoder, print_statement_id);

        let cached_print_statement = cache.print_statement.inner().get(&(crate_id, print_statement_id));

        assert!(cached_print_statement.is_some());

        let expected_format_expression = print_statement.format_expression();

        match decoder.stage {
            FrameDecoderWants::PrintStatement(stage) => {
                let actual_format_expression = stage.segment_decoder.stored_expression;
                assert_eq!(expected_format_expression, actual_format_expression);
            }
            _ => panic!("unexpected stage"),
        }
    }

    #[test]
    fn print_statement_id_not_found_error() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let print_statement_id = PrintStatementId::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u16(*print_statement_id.as_ref());

        let error = decoder.decode(&mut bytes).unwrap_err();
        assert!(matches!(error, RedefmtDecoderError::UnknownStatement(_, _, _)));
    }

    #[test]
    fn full() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let (print_statement_id, print_statement) = seed_print_statement(&decoder, crate_id);
        put_and_decode_print_statement_id(&mut decoder, print_statement_id);

        let value = true;
        let actual_frame = put_and_decode_bool_arg(&mut decoder, value).unwrap();

        let expected_frame = RedefmtFrame::new(None, None, &print_statement, vec![Value::Boolean(value)]);

        assert_eq!(expected_frame, actual_frame);

        // Clears per frame content
        assert!(matches!(decoder.stage, FrameDecoderWants::Header));
    }

    fn seed_crate(decoder: &RedefmtDecoder) -> CrateId {
        let crate_name = CrateName::new("x").unwrap();
        let crate_record = Crate::new(crate_name);
        decoder.stores.main_db.insert(&crate_record).unwrap()
    }

    fn seed_print_statement(
        decoder: &RedefmtDecoder,
        crate_id: CrateId,
    ) -> (PrintStatementId, PrintStatement<'static>) {
        let (crate_db, _) = decoder.stores.cache.krate.inner().get(&crate_id).unwrap();

        let statement = mock_print_statement();
        let id = crate_db.insert(&statement).unwrap();

        (id, statement)
    }

    fn mock_stamp_stage<'a>() -> FrameDecoderWants<'a> {
        FrameDecoderWants::PrintCrateId(WantsPrintCrateIdStage { header: Header::new(false, None), stamp: None })
    }

    fn mock_print_statement() -> PrintStatement<'static> {
        let location = Location::new("file.rs", 1);

        // NOTE: Two format arguments, but only one provided. Implicitly
        // ensures that it only needs to be encoded and decoded once
        let format_expression = StoredFormatExpression::new(deferred_format_expression!("{0} {0}", y = y), false);

        PrintStatement::new(location, format_expression)
    }

    fn put_and_decode_print_crate_id(decoder: &mut RedefmtDecoder, crate_id: CrateId) {
        let mut bytes = BytesMut::new();
        bytes.put_u16(*crate_id.as_ref());
        decoder.decode(&mut bytes).unwrap();
    }

    fn put_and_decode_print_statement_id(decoder: &mut RedefmtDecoder, print_statement_id: PrintStatementId) {
        let mut bytes = BytesMut::new();
        bytes.put_u16(*print_statement_id.as_ref());
        decoder.decode(&mut bytes).unwrap();
    }

    fn put_and_decode_bool_arg<'a>(decoder: &mut RedefmtDecoder<'a>, value: bool) -> Option<RedefmtFrame<'a>> {
        let mut dispatcher = SimpleTestDispatcher::default();
        value.write_value(&mut dispatcher);
        decoder.decode(&mut dispatcher.bytes).unwrap()
    }
}
