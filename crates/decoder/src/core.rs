use redefmt_common::{
    codec::frame::{Header, Stamp},
    identifiers::{CrateId, PrintStatementId},
};
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

pub struct RedefmtDecoder<'cache> {
    // frame indenpendent state
    stores: DecoderStores<'cache>,
    // reset per frame
    stage: DecoderWants<'cache>,
}

impl<'cache> RedefmtDecoder<'cache> {
    pub fn new(stores: DecoderStores<'cache>) -> Self {
        Self { stores, stage: DecoderWants::Header }
    }
}

impl<'cache> tokio_util::codec::Decoder for RedefmtDecoder<'cache> {
    type Error = RedefmtDecoderError;
    type Item = RedefmtFrame<'cache>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let current_stage = std::mem::take(&mut self.stage);
        match current_stage {
            DecoderWants::Header => {
                let Ok(header_byte) = src.try_get_u8() else {
                    return Ok(None);
                };

                let header = Header::from_bits(header_byte).ok_or(RedefmtDecoderError::UnknownHeader(header_byte))?;

                let next_stage = match header.contains(Header::STAMP) {
                    true => DecoderWants::Stamp(WantsStampStage { header }),
                    false => DecoderWants::PrintCrateId(WantsPrintCrateIdStage { header, stamp: None }),
                };

                self.stage = next_stage;
                self.decode(src)
            }
            DecoderWants::Stamp(stage) => {
                let Ok(stamp) = src.try_get_u64().map(Stamp::new) else {
                    self.stage = DecoderWants::Stamp(stage);
                    return Ok(None);
                };

                self.stage =
                    DecoderWants::PrintCrateId(WantsPrintCrateIdStage { header: stage.header, stamp: Some(stamp) });
                self.decode(src)
            }
            DecoderWants::PrintCrateId(stage) => {
                let Ok(print_crate_id) = src.try_get_u16().map(CrateId::new) else {
                    self.stage = DecoderWants::PrintCrateId(stage);
                    return Ok(None);
                };

                let print_crate = self.stores.get_or_insert_crate(print_crate_id)?;

                self.stage = stage.next(print_crate);
                self.decode(src)
            }
            DecoderWants::PrintStatementId(stage) => {
                let Ok(print_statement_id) = src.try_get_u16().map(PrintStatementId::new) else {
                    self.stage = DecoderWants::PrintStatementId(stage);
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
            DecoderWants::PrintStatement(mut stage) => {
                if stage.segment_decoder.decode(&self.stores, src)?.is_none() {
                    self.stage = DecoderWants::PrintStatement(stage);
                    return Ok(None);
                }

                let item = RedefmtFrame {
                    stamp: stage.stamp,
                    print_info: stage.print_info,
                    segments: stage.segment_decoder.decoded_segments,
                };

                self.stage = DecoderWants::Header;

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
        pub fn mock(cache: &'cache DecoderCache) -> (TempDir, Self) {
            let (temp_dir, stores) = DecoderStores::mock(cache);

            let decoder = RedefmtDecoder::new(stores);

            (temp_dir, decoder)
        }
    }
}

#[cfg(test)]
mod tests {
    use redefmt_args::FormatOptions;
    use redefmt_common::codec::encoding::{SimpleTestDispatcher, WriteValue};
    use redefmt_db::{
        Table,
        crate_table::{Crate, CrateName},
        location,
        statement_table::{
            Segment,
            print::{LogLevel, PrintInfo, PrintStatement},
        },
    };
    use tokio_util::{bytes::BufMut, codec::Decoder};

    use super::*;

    #[test]
    fn header() {
        let cache = DecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        assert!(matches!(decoder.stage, DecoderWants::Header));

        let expected_header = Header::all();

        let mut bytes = BytesMut::from_iter([expected_header.bits()]);

        decoder.decode(&mut bytes).unwrap();

        assert!(bytes.is_empty());

        match decoder.stage {
            DecoderWants::Stamp(stage) => {
                assert_eq!(expected_header, stage.header);
            }
            _ => panic!("unexpected stage"),
        }
    }

    #[test]
    fn stamp() {
        let cache = DecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        let header = Header::STAMP;
        let stamp = Stamp::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u8(header.bits());
        bytes.put_u64(*stamp.as_ref());

        decoder.decode(&mut bytes).unwrap();

        assert!(bytes.is_empty());

        match decoder.stage {
            DecoderWants::PrintCrateId(stage) => {
                let actual_stamp = stage.stamp.unwrap();
                assert_eq!(stamp, actual_stamp);
            }
            _ => panic!("unexpected stage"),
        }
    }

    #[test]
    fn print_crate_id() {
        let cache = DecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);

        assert!(cache.krate.inner().is_empty());

        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let expected_crate_value = cache.krate.inner().get(&crate_id).unwrap();

        match decoder.stage {
            DecoderWants::PrintStatementId(stage) => {
                assert_eq!(&expected_crate_value.1, stage.print_crate.record);
            }
            _ => panic!("unexpected stage"),
        }
    }

    #[test]
    fn print_crate_id_not_found_error() {
        let cache = DecoderCache::default();
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
        let cache = DecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn print_statement_id() {
        let cache = DecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let (print_statement_id, print_statement) = seed_print_statement(&decoder, crate_id);

        assert!(!matches!(decoder.stage, DecoderWants::PrintStatement(_)));
        assert!(cache.print_statement.inner().is_empty());

        put_and_decode_print_statement_id(&mut decoder, print_statement_id);

        let cached_print_statement = cache.print_statement.inner().get(&(crate_id, print_statement_id));

        assert!(cached_print_statement.is_some());

        let expected_segments = print_statement.segments().iter().collect::<Vec<_>>();

        match decoder.stage {
            DecoderWants::PrintStatement(stage) => {
                let actual_segments = stage.segment_decoder.segments.collect::<Vec<_>>();
                assert_eq!(expected_segments, actual_segments);
            }
            _ => panic!("unexpected stage"),
        }
    }

    #[test]
    fn print_statement_id_not_found_error() {
        let cache = DecoderCache::default();
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
        let cache = DecoderCache::default();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let (print_statement_id, print_statement) = seed_print_statement(&decoder, crate_id);
        put_and_decode_print_statement_id(&mut decoder, print_statement_id);

        let value = true;
        let actual_frame = put_and_decode_bool_arg(&mut decoder, value).unwrap();

        let format_options = FormatOptions::default();
        let expected_frame = RedefmtFrame {
            stamp: None,
            print_info: print_statement.info(),
            segments: vec![
                DecodedSegment::Value(ComplexValue::Value(Value::Boolean(value)), &format_options),
                DecodedSegment::Str("x"),
            ],
        };

        assert_eq!(expected_frame, actual_frame);

        // Clears per frame content
        assert!(matches!(decoder.stage, DecoderWants::Header));
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

    fn mock_stamp_stage<'a>() -> DecoderWants<'a> {
        DecoderWants::PrintCrateId(WantsPrintCrateIdStage { header: Header::new(false), stamp: None })
    }

    fn mock_print_statement() -> PrintStatement<'static> {
        let print_info = PrintInfo::new(Some(LogLevel::Debug), location!());
        PrintStatement::new(
            print_info,
            vec![Segment::Arg(FormatOptions::default()), Segment::Str("x".into())],
        )
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
