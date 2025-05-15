use std::path::PathBuf;

use encode_unicode::CharExt;
use redefmt_args::FormatOptions;
use redefmt_common::{
    codec::frame::{Header, PointerWidth, Stamp, TypeHint},
    identifiers::{CrateId, PrintStatementId},
};
use redefmt_db::{
    DbClient, MainDb,
    state_dir::StateDir,
    statement_table::{Segment, print::PrintInfo},
};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

use crate::*;

pub struct RedefmtDecoder {
    state_dir: PathBuf,
    main_db: DbClient<MainDb>,
    crate_cache: CrateCache,
    print_statement_cache: PrintStatementCache,

    // Per frame state:
    /// stage 1:
    header: Option<Header>,
    /// stage 2:
    stamp: Option<Stamp>,
    /// stage 3:
    print_crate_id: Option<CrateId>,
    /// stage 4:
    // IMPROVEMENT: use Arc
    print_statement_info: Option<PrintInfo<'static>>,
    /// In reverse order to make taking next cheap
    print_statement_segments: Vec<Segment<'static>>,
    decoded_segments: Vec<DecodedSegment>,
    /// stage 5:
    next_value: Option<(TypeHint, FormatOptions<'static>)>,
    value_context: ValueContext,
}

impl RedefmtDecoder {
    pub fn new() -> Result<Self, RedefmtDecoderError> {
        let dir = StateDir::resolve()?;
        Self::new_impl(dir)
    }

    pub(crate) fn new_impl(db_dir: PathBuf) -> Result<Self, RedefmtDecoderError> {
        let main_db = DbClient::new_main(&db_dir)?;

        Ok(Self {
            state_dir: db_dir,
            main_db,
            crate_cache: Default::default(),
            print_statement_cache: Default::default(),
            header: None,
            stamp: None,
            print_crate_id: None,
            print_statement_info: None,
            print_statement_segments: Default::default(),
            decoded_segments: Default::default(),
            next_value: None,
            value_context: Default::default(),
        })
    }
}

impl tokio_util::codec::Decoder for RedefmtDecoder {
    type Error = RedefmtDecoderError;
    type Item = RedefmtFrame;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let header = match self.header {
            Some(header) => header,
            None => {
                let Ok(header_byte) = src.try_get_u8() else {
                    return Ok(None);
                };

                let header = Header::from_bits(header_byte).ok_or(RedefmtDecoderError::UnknownHeader(header_byte))?;

                self.header = Some(header);

                header
            }
        };

        // stamp
        {
            if header.contains(Header::STAMP) && self.stamp.is_none() {
                let Ok(stamp) = src.try_get_u64().map(Stamp::new) else {
                    return Ok(None);
                };

                self.stamp = Some(stamp);
            }
        }

        let crate_value = match self.print_crate_id {
            Some(crate_id) => {
                // IMPROVEMENT: avoid repeating the gets?
                self.crate_cache.get(crate_id).expect("crate not saved")
            }
            None => {
                let Ok(crate_id) = src.try_get_u16().map(CrateId::new) else {
                    return Ok(None);
                };

                self.print_crate_id = Some(crate_id);

                self.crate_cache
                    .fetch_and_save(crate_id, &self.main_db, &self.state_dir)?
            }
        };

        let print_statement_info = match &self.print_statement_info {
            // IMPROVEMENT: remove clone
            Some(info) => info.clone(),
            None => {
                let Ok(print_statement_id) = src.try_get_u16().map(PrintStatementId::new) else {
                    return Ok(None);
                };

                let print_statement = self
                    .print_statement_cache
                    .fetch_and_save(print_statement_id, crate_value)?;

                // IMPROVEMENT: remove clones
                let info = print_statement.info().clone();
                self.print_statement_info = Some(info.clone());

                // IMPROVEMENT: remove clone
                self.print_statement_segments = print_statement.segments().iter().rev().map(Clone::clone).collect();

                info
            }
        };

        // Always skipped until self.print_statement_segments.pop() returns an Arg which remains to be read.
        if let Some((type_hint, format_options)) = self.next_value.take() {
            match decode_value(type_hint, src, header.pointer_width(), &mut self.value_context)? {
                Some(value) => {
                    self.decoded_segments.push(DecodedSegment::Value(value, format_options));
                    self.value_context = Default::default();
                }
                None => {
                    self.next_value = Some((type_hint, format_options));
                    return Ok(None);
                }
            }
        }

        while let Some(next_segment) = self.print_statement_segments.pop() {
            match next_segment {
                Segment::Str(cow) => {
                    // IMPROVEMENT: use Arc in print_statement_segments and simply clone the reference count here
                    self.decoded_segments.push(DecodedSegment::Str(cow.to_string()));
                }
                Segment::Arg(format_options) => {
                    let Ok(type_hint_repr) = src.try_get_u8() else {
                        self.print_statement_segments.push(Segment::Arg(format_options));
                        return Ok(None);
                    };

                    let type_hint = TypeHint::from_repr(type_hint_repr)
                        .ok_or(RedefmtDecoderError::UnknownTypeHint(type_hint_repr))?;

                    match decode_value(type_hint, src, header.pointer_width(), &mut self.value_context)? {
                        Some(value) => {
                            self.decoded_segments.push(DecodedSegment::Value(value, format_options));
                            self.value_context = Default::default();
                        }
                        None => {
                            self.next_value = Some((type_hint, format_options));
                            return Ok(None);
                        }
                    }
                }
            }
        }

        let item = RedefmtFrame {
            stamp: self.stamp.take(),
            print_info: print_statement_info,
            segments: std::mem::take(&mut self.decoded_segments),
        };

        // Clear remaining frame value
        self.header = None;
        self.print_crate_id = None;
        self.print_statement_info = None;
        self.value_context = Default::default();

        Ok(Some(item))
    }
}

/// Returns no if `src` does not yet contain enough bytes for the given type hint.
/// `value_context` must be cleared by callee if function returns Ok(Some(value))
fn decode_value(
    type_hint: TypeHint,
    src: &mut BytesMut,
    pointer_width: PointerWidth,
    value_context: &mut ValueContext,
) -> Result<Option<Value>, RedefmtDecoderError> {
    match type_hint {
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

            Ok(Some(Value::Boolean(boolean)))
        }
        TypeHint::U8 => Ok(src.try_get_u8().ok().map(Value::U8)),
        TypeHint::U16 => Ok(src.try_get_u16().ok().map(Value::U16)),
        TypeHint::U32 => Ok(src.try_get_u32().ok().map(Value::U32)),
        TypeHint::U64 => Ok(src.try_get_u64().ok().map(Value::U64)),
        TypeHint::U128 => Ok(src.try_get_u128().ok().map(Value::U128)),
        TypeHint::I8 => Ok(src.try_get_i8().ok().map(Value::I8)),
        TypeHint::I16 => Ok(src.try_get_i16().ok().map(Value::I16)),
        TypeHint::I32 => Ok(src.try_get_i32().ok().map(Value::I32)),
        TypeHint::I64 => Ok(src.try_get_i64().ok().map(Value::I64)),
        TypeHint::F32 => Ok(src.try_get_f32().ok().map(Value::F32)),
        TypeHint::F64 => Ok(src.try_get_f64().ok().map(Value::F64)),
        TypeHint::Usize => get_pointer_width_num(src, pointer_width)
            .map(|num| Ok(Value::Usize(num)))
            .transpose(),
        TypeHint::Isize => {
            if src.len() < pointer_width.size() {
                return Ok(None);
            }

            let num = match pointer_width {
                PointerWidth::U16 => src.get_i16() as i64,
                PointerWidth::U32 => src.get_i32() as i64,
                PointerWidth::U64 => src.get_i64(),
            };

            Ok(Some(Value::Isize(num)))
        }
        TypeHint::Char => {
            let Ok(char_length) = src.try_get_u8() else {
                return Ok(None);
            };

            if src.len() < char_length as usize {
                return Ok(None);
            }

            let utf8_bytes = match char_length {
                1 => [src.get_u8(), 0, 0, 0],
                2 => [src.get_u8(), src.get_u8(), 0, 0],
                3 => [src.get_u8(), src.get_u8(), src.get_u8(), 0],
                4 => [src.get_u8(), src.get_u8(), src.get_u8(), src.get_u8()],
                n => return Err(RedefmtDecoderError::InvalidCharLength(n)),
            };

            let char = char::from_utf8_array(utf8_bytes)?;

            Ok(Some(Value::Char(char)))
        }
        TypeHint::StringSlice => {
            let length = match value_context.length {
                Some(length) => length,
                None => {
                    let Some(length) = get_pointer_width_num(src, pointer_width) else {
                        return Ok(None);
                    };

                    value_context.length = Some(length);

                    length
                }
            };

            let length = usize::try_from(length).map_err(|_| RedefmtDecoderError::LengthOverflow(length))?;

            if src.len() < length {
                return Ok(None);
            }

            let mut vec = Vec::with_capacity(length);
            vec.put(src.take(length));

            let string = String::from_utf8(vec)?;

            Ok(Some(Value::String(string)))
        }
        TypeHint::List => todo!(),
        TypeHint::DynList => todo!(),
        TypeHint::Tuple => todo!(),
        TypeHint::WriteId => todo!(),
        // TODO:
        // TypeHint::Set => todo!(),
        // TypeHint::Map => todo!(),
        // TypeHint::DynMap => todo!(),
    }
}

fn get_pointer_width_num(src: &mut BytesMut, pointer_width: PointerWidth) -> Option<u64> {
    if src.len() < pointer_width.size() {
        return None;
    }

    let num = match pointer_width {
        PointerWidth::U16 => src.get_u16() as u64,
        PointerWidth::U32 => src.get_u32() as u64,
        PointerWidth::U64 => src.get_u64(),
    };

    Some(num)
}

#[cfg(test)]
mod mock {
    use tempfile::TempDir;

    use super::*;

    impl RedefmtDecoder {
        pub fn mock() -> (TempDir, Self) {
            let temp_dir = tempfile::tempdir().unwrap();

            let decoder = RedefmtDecoder::new_impl(temp_dir.path().to_path_buf()).unwrap();

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
            let mut bytes = BytesMut::from_iter([2]);
            let error = decode_value(
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
            let invalid_utf8_bytes = [0xE0, 0x80, 0x80];
            let mut bytes = BytesMut::new();
            bytes.put_u8(invalid_utf8_bytes.len() as u8);
            bytes.put_slice(&invalid_utf8_bytes);

            let error = decode_value(
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
            let invalid_utf8_bytes = [0xE0, 0x80, 0x80];
            let mut bytes = BytesMut::new();
            bytes.put_slice(&invalid_utf8_bytes.len().to_be_bytes());
            bytes.put_slice(&invalid_utf8_bytes);

            let error = decode_value(
                TypeHint::StringSlice,
                &mut bytes,
                PointerWidth::of_target(),
                &mut Default::default(),
            )
            .unwrap_err();

            assert!(matches!(error, RedefmtDecoderError::InvalidStringBytes(_)))
        }
    }

    fn assert_value<T: WriteValue>(type_hint: TypeHint, encoded_value: T, from_inner: fn(T) -> Value) {
        let mut dispatcher = SimpleTestDispatcher::default();
        encoded_value.write_value(&mut dispatcher);

        let mut dispatched_bytes = dispatcher.bytes;

        let dispatched_type_hint = TypeHint::from_repr(dispatched_bytes.get_u8()).unwrap();

        assert_eq!(dispatched_type_hint, type_hint);

        let actual_value = decode_value(
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
    fn empty_after_new() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn header() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        let expected_header = Header::all();

        let mut bytes = BytesMut::from_iter([expected_header.bits()]);

        assert!(decoder.header.is_none());

        decoder.decode(&mut bytes).unwrap();
        assert!(bytes.is_empty());

        assert_eq!(expected_header, decoder.header.unwrap());
    }

    #[test]
    fn empty_after_header() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::all());

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn stamp() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        let header = Header::STAMP;
        let stamp = Stamp::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u8(header.bits());
        bytes.put_u64(*stamp.as_ref());

        assert!(decoder.stamp.is_none());

        decoder.decode(&mut bytes).unwrap();
        assert!(bytes.is_empty());

        assert_eq!(stamp, decoder.stamp.unwrap());
    }

    #[test]
    fn empty_after_stamp() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn print_crate_id() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let crate_id = seed_crate(&mut decoder);

        assert!(decoder.print_crate_id.is_none());
        assert!(decoder.crate_cache.inner().is_empty());

        put_and_decode_print_crate_id(&mut decoder, crate_id);

        assert!(decoder.print_crate_id.is_some_and(|id| id == crate_id));
        assert!(decoder.crate_cache.inner().contains_key(&crate_id));
    }

    #[test]
    fn print_crate_id_not_found_error() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let crate_id = CrateId::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u16(*crate_id.as_ref());

        let error = decoder.decode(&mut bytes).unwrap_err();
        assert!(matches!(error, RedefmtDecoderError::UnknownCrate(_)));
    }

    #[test]
    fn empty_after_print_crate_id() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let crate_id = seed_crate(&mut decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn print_statement_id() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let crate_id = seed_crate(&mut decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let (print_statement_id, print_statement) = seed_print_statement(&mut decoder);

        assert!(decoder.print_statement_segments.is_empty());
        assert!(decoder.print_statement_cache.inner().is_empty());

        put_and_decode_print_statement_id(&mut decoder, print_statement_id);

        let expected_segments = print_statement.segments();
        let mut actual_segments = decoder.print_statement_segments;
        actual_segments.reverse();

        assert_eq!(expected_segments, &actual_segments);

        assert!(decoder.print_statement_cache.inner().contains_key(&print_statement_id));
    }

    #[test]
    fn print_statement_id_not_found_error() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let crate_id = seed_crate(&mut decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let print_statement_id = PrintStatementId::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u16(*print_statement_id.as_ref());

        let error = decoder.decode(&mut bytes).unwrap_err();
        assert!(matches!(error, RedefmtDecoderError::UnknownPrintStatement(_, _)));
    }

    #[test]
    fn full() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let crate_id = seed_crate(&mut decoder);
        put_and_decode_print_crate_id(&mut decoder, crate_id);

        let (print_statement_id, print_statement) = seed_print_statement(&mut decoder);
        put_and_decode_print_statement_id(&mut decoder, print_statement_id);

        let value = true;
        let actual_frame = put_and_decode_bool_arg(&mut decoder, value).unwrap();

        let expected_frame = RedefmtFrame {
            stamp: None,
            print_info: print_statement.info().clone(),
            segments: vec![
                DecodedSegment::Value(Value::Boolean(value), FormatOptions::default()),
                DecodedSegment::Str("x".to_string()),
            ],
        };

        assert_eq!(expected_frame, actual_frame);

        // Clears per frame content
        assert!(decoder.header.is_none());
        assert!(decoder.stamp.is_none());
        assert!(decoder.print_crate_id.is_none());
        assert!(decoder.print_statement_info.is_none());
        assert!(decoder.print_statement_segments.is_empty());
        assert!(decoder.decoded_segments.is_empty());
        assert!(decoder.next_value.is_none());
    }

    fn seed_crate(decoder: &mut RedefmtDecoder) -> CrateId {
        let crate_name = CrateName::new("x").unwrap();
        let crate_record = Crate::new(crate_name);
        decoder.main_db.insert(&crate_record).unwrap()
    }

    fn seed_print_statement(decoder: &mut RedefmtDecoder) -> (PrintStatementId, PrintStatement<'static>) {
        let crate_id = decoder.print_crate_id.unwrap();
        let (crate_db, _) = decoder.crate_cache.get(crate_id).unwrap();

        let statement = mock_print_statement();
        let id = crate_db.insert(&statement).unwrap();

        (id, statement)
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

    fn put_and_decode_bool_arg(decoder: &mut RedefmtDecoder, value: bool) -> Option<RedefmtFrame> {
        let mut dispatcher = SimpleTestDispatcher::default();
        value.write_value(&mut dispatcher);
        decoder.decode(&mut dispatcher.bytes).unwrap()
    }
}
