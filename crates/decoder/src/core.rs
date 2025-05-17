use encode_unicode::CharExt;
use redefmt_common::{
    codec::frame::{Header, PointerWidth, Stamp, TypeHint},
    identifiers::{CrateId, PrintStatementId},
};
use redefmt_db::statement_table::Segment;
use tokio_util::bytes::{Buf, BufMut, BytesMut};

use crate::*;

pub struct RedefmtDecoder<'caches> {
    // frame indenpendent state
    stores: DecoderStores<'caches>,
    // reset per frame
    stage: DecoderWants<'caches>,
}

impl<'caches> RedefmtDecoder<'caches> {
    pub fn new(stores: DecoderStores<'caches>) -> Self {
        Self { stores, stage: DecoderWants::Header }
    }
}

impl<'caches> tokio_util::codec::Decoder for RedefmtDecoder<'caches> {
    type Error = RedefmtDecoderError;
    type Item = RedefmtFrame<'caches>;

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

                let print_crate_value = self.stores.get_or_insert_crate(print_crate_id)?;

                self.stage = stage.next(print_crate_value);
                self.decode(src)
            }
            DecoderWants::PrintStatementId(stage) => {
                let Ok(print_statement_id) = src.try_get_u16().map(PrintStatementId::new) else {
                    self.stage = DecoderWants::PrintStatementId(stage);
                    return Ok(None);
                };

                let print_statement = self
                    .stores
                    .print_statement_cache
                    .get_or_insert(print_statement_id, stage.print_crate_value)?;

                self.stage = stage.next(print_statement);
                self.decode(src)
            }
            DecoderWants::PrintStatement(mut stage) => {
                if decode_segments(&self.stores, &mut stage, src)?.is_none() {
                    self.stage = DecoderWants::PrintStatement(stage);
                    return Ok(None);
                }

                let item = RedefmtFrame {
                    stamp: stage.stamp,
                    print_info: stage.print_info,
                    segments: stage.decoded_segments,
                };

                self.stage = DecoderWants::Header;

                Ok(Some(item))
            }
        }
    }
}

fn decode_segments<'caches>(
    stores: &DecoderStores<'caches>,
    stage: &mut WantsPrintStatementStage<'caches>,
    src: &mut BytesMut,
) -> Result<Option<()>, RedefmtDecoderError> {
    if let Some(current_value_context) = stage.segment_context.current_value.take() {
        let SegmentValueContext { type_hint, format_options, mut value_context } = current_value_context;

        match decode_value(stores, type_hint, src, stage.header.pointer_width(), &mut value_context)? {
            Some(value) => {
                stage
                    .decoded_segments
                    .push(DecodedSegment::Value(value, format_options));
                stage.segment_context.current_value = None;
            }
            None => {
                stage.segment_context.current_value =
                    Some(SegmentValueContext { type_hint, format_options, value_context });
                return Ok(None);
            }
        }
    }

    while let Some(next_segment) = stage.segment_context.segments.peek() {
        match next_segment {
            Segment::Str(str) => {
                stage.decoded_segments.push(DecodedSegment::Str(str));
                stage.segment_context.segments.next();
            }
            Segment::Arg(format_options) => {
                let Ok(type_hint_repr) = src.try_get_u8() else {
                    return Ok(None);
                };

                let type_hint =
                    TypeHint::from_repr(type_hint_repr).ok_or(RedefmtDecoderError::UnknownTypeHint(type_hint_repr))?;

                let mut value_context = ValueContext::default();

                match decode_value(stores, type_hint, src, stage.header.pointer_width(), &mut value_context)? {
                    Some(value) => {
                        stage
                            .decoded_segments
                            .push(DecodedSegment::Value(value, format_options));
                        stage.segment_context.segments.next();
                    }
                    None => {
                        stage.segment_context.current_value =
                            Some(SegmentValueContext { type_hint, format_options, value_context });
                        return Ok(None);
                    }
                }
            }
        }
    }

    Ok(Some(()))
}

/// Returns no if `src` does not yet contain enough bytes for the given type hint.
/// `value_context` must be cleared by callee if function returns Ok(Some(value))
fn decode_value<'caches>(
    stores: &DecoderStores<'caches>,
    type_hint: TypeHint,
    src: &mut BytesMut,
    pointer_width: PointerWidth,
    value_context: &mut ValueContext,
) -> Result<Option<Value>, RedefmtDecoderError> {
    let maybe_value = match type_hint {
        TypeHint::U8 => src.try_get_u8().ok().map(Value::U8),
        TypeHint::U16 => src.try_get_u16().ok().map(Value::U16),
        TypeHint::U32 => src.try_get_u32().ok().map(Value::U32),
        TypeHint::U64 => src.try_get_u64().ok().map(Value::U64),
        TypeHint::U128 => src.try_get_u128().ok().map(Value::U128),
        TypeHint::I8 => src.try_get_i8().ok().map(Value::I8),
        TypeHint::I16 => src.try_get_i16().ok().map(Value::I16),
        TypeHint::I32 => src.try_get_i32().ok().map(Value::I32),
        TypeHint::I64 => src.try_get_i64().ok().map(Value::I64),
        TypeHint::F32 => src.try_get_f32().ok().map(Value::F32),
        TypeHint::F64 => src.try_get_f64().ok().map(Value::F64),
        TypeHint::Usize => DecoderUtils::get_target_usize(src, pointer_width).map(Value::Usize),
        TypeHint::Isize => DecoderUtils::get_target_isize(src, pointer_width).map(Value::Isize),
        TypeHint::Boolean => {
            let Ok(bool_byte) = src.try_get_u8() else {
                return Ok(None);
            };

            let boolean = match bool_byte {
                val if val == true as u8 => true,
                val if val == false as u8 => false,
                _ => {
                    return Err(RedefmtDecoderError::InvalidValueBytes(type_hint, vec![bool_byte]));
                }
            };

            Some(Value::Boolean(boolean))
        }
        TypeHint::Char => {
            let Some(length) = value_context.get_or_store_u8_length(src) else {
                return Ok(None);
            };

            if src.len() < length {
                return Ok(None);
            }

            let utf8_bytes = match length {
                1 => [src.get_u8(), 0, 0, 0],
                2 => [src.get_u8(), src.get_u8(), 0, 0],
                3 => [src.get_u8(), src.get_u8(), src.get_u8(), 0],
                4 => [src.get_u8(), src.get_u8(), src.get_u8(), src.get_u8()],
                n => return Err(RedefmtDecoderError::InvalidCharLength(n as u8)),
            };

            let char = char::from_utf8_array(utf8_bytes)?;

            Some(Value::Char(char))
        }
        TypeHint::StringSlice => {
            let Some(length) = value_context.get_or_store_usize_length(src, pointer_width)? else {
                return Ok(None);
            };

            if src.len() < length {
                return Ok(None);
            }

            let mut vec = Vec::with_capacity(length);
            vec.put(src.take(length));

            let string = String::from_utf8(vec)?;

            Some(Value::String(string))
        }
        TypeHint::Tuple => {
            let Some(length) = value_context.get_or_store_u8_length(src) else {
                return Ok(None);
            };

            let Some(values) = decode_dyn_list(stores, src, pointer_width, value_context, length)? else {
                return Ok(None);
            };

            Some(Value::Tuple(values))
        }
        TypeHint::DynList => {
            let Some(length) = value_context.get_or_store_usize_length(src, pointer_width)? else {
                return Ok(None);
            };

            let Some(values) = decode_dyn_list(stores, src, pointer_width, value_context, length)? else {
                return Ok(None);
            };

            Some(Value::List(values))
        }
        TypeHint::List => decode_list(stores, src, pointer_width, value_context)?.map(Value::List),
        TypeHint::WriteId => {
            let write_context = value_context.write_context.get_or_insert_default();

            let Some(crate_id) = write_context.get_or_store_crate_id(src) else {
                return Ok(None);
            };

            let crate_value = stores.get_or_insert_crate(crate_id)?;

            // TODO:

            let Some(statement_id) = write_context.get_or_store_statement_id(src) else {
                return Ok(None);
            };

            todo!();
        } /* TODO:
           * TypeHint::Set => todo!(),
           * TypeHint::Map => todo!(),
           * TypeHint::DynMap => todo!(), */
    };

    Ok(maybe_value)
}

fn decode_dyn_list<'caches>(
    stores: &DecoderStores<'caches>,
    src: &mut BytesMut,
    pointer_width: PointerWidth,
    value_context: &mut ValueContext,
    length: usize,
) -> Result<Option<Vec<Value>>, RedefmtDecoderError> {
    let list_context = value_context
        .list_context
        .get_or_insert_with(|| ListValueContext::new(length));

    while list_context.buffer.len() < length {
        let Some(element_type_hint) = list_context.get_or_insert_element_type_hint(src)? else {
            return Ok(None);
        };

        let element_context = list_context.element_context.get_or_insert_default();

        match decode_value(stores, element_type_hint, src, pointer_width, element_context)? {
            Some(value) => {
                list_context.buffer.push(value);

                // Clear element context
                list_context.element_type_hint = None;
                list_context.element_context = None;
            }
            None => return Ok(None),
        }
    }

    let values = std::mem::take(&mut list_context.buffer);

    // No need to clear value_context.list_value_context, decode value should be done here

    Ok(Some(values))
}

fn decode_list<'caches>(
    stores: &DecoderStores<'caches>,
    src: &mut BytesMut,
    pointer_width: PointerWidth,
    value_context: &mut ValueContext,
) -> Result<Option<Vec<Value>>, RedefmtDecoderError> {
    let Some(length) = value_context.get_or_store_usize_length(src, pointer_width)? else {
        return Ok(None);
    };

    let list_context = value_context
        .list_context
        .get_or_insert_with(|| ListValueContext::new(length));

    let Some(element_type_hint) = list_context.get_or_insert_element_type_hint(src)? else {
        return Ok(None);
    };

    while list_context.buffer.len() < length {
        let element_context = list_context.element_context.get_or_insert_default();

        match decode_value(stores, element_type_hint, src, pointer_width, element_context)? {
            Some(value) => {
                list_context.buffer.push(value);
                list_context.element_context = None;
            }
            None => return Ok(None),
        }
    }

    let values = std::mem::take(&mut list_context.buffer);

    // No need to clear value_context.list_value_context, decode value should be done here

    Ok(Some(values))
}

#[cfg(test)]
mod mock {
    use redefmt_db::statement_table::print::PrintStatement;
    use tempfile::TempDir;

    use super::*;

    impl<'caches> RedefmtDecoder<'caches> {
        pub fn mock(
            crate_cache: &'caches CrateCache,
            print_statement_cache: &'caches StatementCache<PrintStatement<'static>>,
        ) -> (TempDir, Self) {
            let (temp_dir, stores) = DecoderStores::mock(crate_cache, print_statement_cache);

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

    mod decode_value {
        use super::*;

        #[test]
        fn boolean_false() {
            assert_value(TypeHint::Boolean, true, Value::Boolean);
            assert_value(TypeHint::Boolean, false, Value::Boolean);
        }

        #[test]
        fn boolean_err() {
            let crate_cache = CrateCache::new();
            let print_statement_cache = StatementCache::new();
            let (_dir_guard, stores) = DecoderStores::mock(&crate_cache, &print_statement_cache);

            let mut bytes = BytesMut::from_iter([2]);
            let error = decode_value(
                &stores,
                TypeHint::Boolean,
                &mut bytes,
                PointerWidth::of_target(),
                &mut Default::default(),
            )
            .unwrap_err();

            assert!(matches!(error, RedefmtDecoderError::InvalidValueBytes(_, _)));
        }

        #[test]
        fn num() {
            test_decode_int::<usize>(TypeHint::Usize, |inner| Value::Usize(inner as u64));
            test_decode_int::<u8>(TypeHint::U8, Value::U8);
            test_decode_int::<u16>(TypeHint::U16, Value::U16);
            test_decode_int::<u32>(TypeHint::U32, Value::U32);
            test_decode_int::<u64>(TypeHint::U64, Value::U64);
            test_decode_int::<u128>(TypeHint::U128, Value::U128);
            test_decode_int::<isize>(TypeHint::Isize, |inner| Value::Isize(inner as i64));
            test_decode_int::<i8>(TypeHint::I8, Value::I8);
            test_decode_int::<i16>(TypeHint::I16, Value::I16);
            test_decode_int::<i32>(TypeHint::I32, Value::I32);
            test_decode_int::<i64>(TypeHint::I64, Value::I64);
            test_decode_int::<f32>(TypeHint::F32, Value::F32);
            test_decode_int::<f64>(TypeHint::F64, Value::F64);

            fn test_decode_int<T: WriteValue + num_traits::One>(type_hint: TypeHint, from_inner: fn(T) -> Value) {
                assert_value(type_hint, T::one(), from_inner);
            }
        }

        #[test]
        fn char() {
            assert_value(TypeHint::Char, 'x', Value::Char);
            assert_value(TypeHint::Char, 'ß', Value::Char);
            assert_value(TypeHint::Char, 'ᴪ', Value::Char);
            assert_value(TypeHint::Char, '🦀', Value::Char);
        }

        #[test]
        fn char_invalid_utf8_error() {
            let crate_cache = CrateCache::new();
            let print_statement_cache = StatementCache::new();
            let (_dir_guard, stores) = DecoderStores::mock(&crate_cache, &print_statement_cache);

            let invalid_utf8_bytes = [0xE0, 0x80, 0x80];
            let mut bytes = BytesMut::new();
            bytes.put_u8(invalid_utf8_bytes.len() as u8);
            bytes.put_slice(&invalid_utf8_bytes);

            let error = decode_value(
                &stores,
                TypeHint::Char,
                &mut bytes,
                PointerWidth::of_target(),
                &mut Default::default(),
            )
            .unwrap_err();

            assert!(matches!(error, RedefmtDecoderError::InvalidUtf8Char(_)))
        }

        #[test]
        fn string() {
            assert_value(TypeHint::StringSlice, "abc", |str| Value::String(str.to_string()));
            assert_value(TypeHint::StringSlice, "🦀", |str| Value::String(str.to_string()));
        }

        #[test]
        fn string_invalid_utf8_error() {
            let crate_cache = CrateCache::new();
            let print_statement_cache = StatementCache::new();
            let (_dir_guard, stores) = DecoderStores::mock(&crate_cache, &print_statement_cache);

            let invalid_utf8_bytes = [0xE0, 0x80, 0x80];
            let mut bytes = BytesMut::new();
            bytes.put_slice(&invalid_utf8_bytes.len().to_be_bytes());
            bytes.put_slice(&invalid_utf8_bytes);

            let error = decode_value(
                &stores,
                TypeHint::StringSlice,
                &mut bytes,
                PointerWidth::of_target(),
                &mut Default::default(),
            )
            .unwrap_err();

            assert!(matches!(error, RedefmtDecoderError::InvalidStringBytes(_)))
        }

        #[test]
        fn tuple() {
            assert_value(TypeHint::Tuple, (10, "x"), |(num, str)| {
                Value::Tuple(vec![Value::U8(num), Value::String(str.to_string())])
            });
            assert_value(TypeHint::Tuple, ((10, "x"), false), |((num, str), bool)| {
                let inner = Value::Tuple(vec![Value::U8(num), Value::String(str.to_string())]);
                Value::Tuple(vec![inner, Value::Boolean(bool)])
            });
        }

        #[test]
        fn dyn_list() {
            assert_value(TypeHint::DynList, [&10u8 as &dyn WriteValue, &"x"], |_| {
                Value::List(vec![Value::U8(10), Value::String("x".to_string())])
            });
        }

        #[test]
        fn list() {
            assert_value(TypeHint::List, [[0u8, 1], [2, 3]], |_| {
                Value::List(vec![
                    Value::List(vec![Value::U8(0), Value::U8(1)]),
                    Value::List(vec![Value::U8(2), Value::U8(3)]),
                ])
            });
        }
    }

    fn assert_value<T: WriteValue>(type_hint: TypeHint, encoded_value: T, from_inner: fn(T) -> Value) {
        let crate_cache = CrateCache::new();
        let print_statement_cache = StatementCache::new();
        let (_dir_guard, stores) = DecoderStores::mock(&crate_cache, &print_statement_cache);

        let mut dispatcher = SimpleTestDispatcher::default();
        encoded_value.write_value(&mut dispatcher);

        let mut dispatched_bytes = dispatcher.bytes;

        let dispatched_type_hint = TypeHint::from_repr(dispatched_bytes.get_u8()).unwrap();

        assert_eq!(dispatched_type_hint, type_hint);

        let actual_value = decode_value(
            &stores,
            type_hint,
            &mut dispatched_bytes,
            PointerWidth::of_target(),
            &mut Default::default(),
        )
        .unwrap()
        .unwrap();

        let expected_value = from_inner(encoded_value);

        assert_eq!(expected_value, actual_value);
    }

    #[test]
    fn header() {
        let crate_cache = CrateCache::new();
        let print_statement_cache = StatementCache::new();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&crate_cache, &print_statement_cache);

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
        let crate_cache = CrateCache::new();
        let print_statement_cache = StatementCache::new();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&crate_cache, &print_statement_cache);

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
        let crate_cache = CrateCache::new();
        let print_statement_cache = StatementCache::new();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&crate_cache, &print_statement_cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);

        assert!(crate_cache.inner().is_empty());

        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let expected_crate_value = decoder.stores.crate_cache.inner().get(&crate_id).unwrap();

        match decoder.stage {
            DecoderWants::PrintStatementId(stage) => {
                assert_eq!(expected_crate_value.1, stage.print_crate_value.1);
            }
            _ => panic!("unexpected stage"),
        }
    }

    #[test]
    fn print_crate_id_not_found_error() {
        let crate_cache = CrateCache::new();
        let print_statement_cache = StatementCache::new();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&crate_cache, &print_statement_cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = CrateId::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u16(*crate_id.as_ref());

        let error = decoder.decode(&mut bytes).unwrap_err();
        assert!(matches!(error, RedefmtDecoderError::UnknownCrate(_)));
    }

    #[test]
    fn empty_after_print_crate_id() {
        let crate_cache = CrateCache::new();
        let print_statement_cache = StatementCache::new();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&crate_cache, &print_statement_cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn print_statement_id() {
        let crate_cache = CrateCache::new();
        let print_statement_cache = StatementCache::new();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&crate_cache, &print_statement_cache);

        decoder.stage = mock_stamp_stage();

        let crate_id = seed_crate(&decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let (print_statement_id, print_statement) = seed_print_statement(&decoder, crate_id);

        assert!(!matches!(decoder.stage, DecoderWants::PrintStatement(_)));
        assert!(decoder.stores.print_statement_cache.inner().is_empty());

        put_and_decode_print_statement_id(&mut decoder, print_statement_id);

        let cached_print_statement = decoder.stores.print_statement_cache.inner().get(&print_statement_id);
        assert!(cached_print_statement.is_some());

        let expected_segments = print_statement.segments().iter().collect::<Vec<_>>();

        match decoder.stage {
            DecoderWants::PrintStatement(stage) => {
                let actual_segments = stage.segment_context.segments.collect::<Vec<_>>();
                assert_eq!(expected_segments, actual_segments);
            }
            _ => panic!("unexpected stage"),
        }
    }

    #[test]
    fn print_statement_id_not_found_error() {
        let crate_cache = CrateCache::new();
        let print_statement_cache = StatementCache::new();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&crate_cache, &print_statement_cache);

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
        let crate_cache = CrateCache::new();
        let print_statement_cache = StatementCache::new();
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock(&crate_cache, &print_statement_cache);

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
                DecodedSegment::Value(Value::Boolean(value), &format_options),
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
        let (crate_db, _) = decoder.stores.crate_cache.inner().get(&crate_id).unwrap();

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
