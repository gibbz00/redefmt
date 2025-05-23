use encode_unicode::CharExt;
use redefmt_internal::{
    codec::frame::{PointerWidth, TypeHint},
    identifiers::CrateId,
};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

use crate::*;

pub struct ValueDecoder<'cache> {
    pointer_width: PointerWidth,
    type_hint: TypeHint,
    length_context: Option<usize>,
    list_decoder: Option<ListValueDecoder<'cache>>,
    write_decoder: Option<WriteStatementDecoder<'cache>>,
}

impl<'cache> ValueDecoder<'cache> {
    pub fn new(pointer_width: PointerWidth, type_hint: TypeHint) -> Self {
        Self {
            pointer_width,
            type_hint,
            length_context: None,
            list_decoder: None,
            write_decoder: Default::default(),
        }
    }

    pub fn decode(
        &mut self,
        stores: &Stores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<ComplexValue<'cache>>, RedefmtDecoderError> {
        let maybe_simple_value = match self.type_hint {
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
            TypeHint::Usize => DecoderUtils::get_target_usize(src, self.pointer_width).map(Value::Usize),
            TypeHint::Isize => DecoderUtils::get_target_isize(src, self.pointer_width).map(Value::Isize),
            TypeHint::Boolean => {
                let Ok(bool_byte) = src.try_get_u8() else {
                    return Ok(None);
                };

                let boolean = match bool_byte {
                    val if val == true as u8 => true,
                    val if val == false as u8 => false,
                    _ => {
                        return Err(RedefmtDecoderError::InvalidValueBytes(self.type_hint, vec![bool_byte]));
                    }
                };

                Some(Value::Boolean(boolean))
            }
            TypeHint::Char => {
                let Some(length) = self.get_or_store_u8_length(src) else {
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
                let Some(length) = self.get_or_store_usize_length(src)? else {
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
                let Some(list_decoder) = self.get_or_store_u8_list(src) else {
                    return Ok(None);
                };

                let maybe_tuple = list_decoder.decode_dyn_list(stores, src)?;
                return Ok(maybe_tuple.map(ComplexValue::Tuple));
            }
            TypeHint::DynList => {
                let Some(list_decoder) = self.get_or_store_usize_list(src)? else {
                    return Ok(None);
                };

                let maybe_list = list_decoder.decode_dyn_list(stores, src)?;
                return Ok(maybe_list.map(ComplexValue::List));
            }
            TypeHint::List => {
                let Some(list_decoder) = self.get_or_store_usize_list(src)? else {
                    return Ok(None);
                };

                let maybe_list = list_decoder.decode_list(stores, src)?;
                return Ok(maybe_list.map(ComplexValue::List));
            }
            TypeHint::WriteId => {
                let Some(write_statement_decoder) = self.get_or_store_write_decoder(stores, src)? else {
                    return Ok(None);
                };

                return write_statement_decoder.decode(stores, src);
            } /* TODO:
               * TypeHint::Set => todo!(),
               * TypeHint::Map => todo!(),
               * TypeHint::DynMap => todo!(), */
        };

        Ok(maybe_simple_value.map(ComplexValue::Value))
    }

    fn get_or_store_u8_length(&mut self, src: &mut BytesMut) -> Option<usize> {
        if let Some(length) = self.length_context {
            return Some(length);
        };

        let Ok(length) = src.try_get_u8().map(Into::into) else {
            return None;
        };

        self.length_context = Some(length);

        Some(length)
    }

    fn get_or_store_usize_length(&mut self, src: &mut BytesMut) -> Result<Option<usize>, RedefmtDecoderError> {
        if let Some(length) = self.length_context {
            return Ok(Some(length));
        };

        let Some(length) = self.get_usize(src)? else {
            return Ok(None);
        };

        self.length_context = Some(length);

        Ok(Some(length))
    }

    fn get_or_store_u8_list(&mut self, src: &mut BytesMut) -> Option<&mut ListValueDecoder<'cache>> {
        if self.list_decoder.is_none() {
            let Ok(length) = src.try_get_u8().map(Into::into) else {
                return None;
            };

            self.list_decoder = Some(ListValueDecoder::new(self.pointer_width, length));
        }

        self.list_decoder.as_mut()
    }

    fn get_or_store_usize_list(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<&mut ListValueDecoder<'cache>>, RedefmtDecoderError> {
        if self.list_decoder.is_none() {
            let Some(length) = self.get_usize(src)? else {
                return Ok(None);
            };

            self.list_decoder = Some(ListValueDecoder::new(self.pointer_width, length));
        }

        Ok(self.list_decoder.as_mut())
    }

    fn get_usize(&self, src: &mut BytesMut) -> Result<Option<usize>, RedefmtDecoderError> {
        let Some(length) = DecoderUtils::get_target_usize(src, self.pointer_width) else {
            return Ok(None);
        };

        usize::try_from(length)
            .map(Some)
            .map_err(|_| RedefmtDecoderError::LengthOverflow(length))
    }

    fn get_or_store_write_decoder(
        &mut self,
        stores: &Stores<'cache>,
        src: &mut BytesMut,
    ) -> Result<Option<&mut WriteStatementDecoder<'cache>>, RedefmtDecoderError> {
        if self.write_decoder.is_none() {
            let Ok(crate_id) = src.try_get_u16().map(CrateId::new) else {
                return Ok(None);
            };

            let write_crate = stores.get_or_insert_crate(crate_id)?;

            self.write_decoder = Some(WriteStatementDecoder::new(self.pointer_width, write_crate))
        }

        Ok(self.write_decoder.as_mut())
    }
}

#[cfg(test)]
mod tests {
    use redefmt_args::{FormatString, provided_args::CombinedFormatString};
    use redefmt_db::{
        Table,
        crate_table::{Crate, CrateName},
        statement_table::write::WriteStatement,
    };
    use redefmt_internal::codec::encoding::{SimpleTestDispatcher, WriteValue};

    use super::*;

    #[test]
    fn boolean_false() {
        assert_simple_value(TypeHint::Boolean, true, Value::Boolean);
        assert_simple_value(TypeHint::Boolean, false, Value::Boolean);
    }

    #[test]
    fn boolean_err() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, stores) = Stores::mock(&cache);

        let mut value_decoder = ValueDecoder::new(PointerWidth::of_target(), TypeHint::Boolean);
        let mut bytes = BytesMut::from_iter([2]);

        let error = value_decoder.decode(&stores, &mut bytes).unwrap_err();

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
            assert_simple_value(type_hint, T::one(), from_inner);
        }
    }

    #[test]
    fn char() {
        assert_simple_value(TypeHint::Char, 'x', Value::Char);
        assert_simple_value(TypeHint::Char, 'ß', Value::Char);
        assert_simple_value(TypeHint::Char, 'ᴪ', Value::Char);
        assert_simple_value(TypeHint::Char, '🦀', Value::Char);
    }

    #[test]
    fn char_invalid_utf8_error() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, stores) = Stores::mock(&cache);

        let invalid_utf8_bytes = [0xE0, 0x80, 0x80];
        let mut bytes = BytesMut::new();
        bytes.put_u8(invalid_utf8_bytes.len() as u8);
        bytes.put_slice(&invalid_utf8_bytes);

        let mut value_decoder = ValueDecoder::new(PointerWidth::of_target(), TypeHint::Char);

        let error = value_decoder.decode(&stores, &mut bytes).unwrap_err();

        assert!(matches!(error, RedefmtDecoderError::InvalidUtf8Char(_)))
    }

    #[test]
    fn string() {
        assert_simple_value(TypeHint::StringSlice, "abc", |str| Value::String(str.to_string()));
        assert_simple_value(TypeHint::StringSlice, "🦀", |str| Value::String(str.to_string()));
    }

    #[test]
    fn string_invalid_utf8_error() {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, stores) = Stores::mock(&cache);

        let invalid_utf8_bytes = [0xE0, 0x80, 0x80];
        let mut bytes = BytesMut::new();
        bytes.put_slice(&invalid_utf8_bytes.len().to_be_bytes());
        bytes.put_slice(&invalid_utf8_bytes);

        let mut value_decoder = ValueDecoder::new(PointerWidth::of_target(), TypeHint::StringSlice);

        let error = value_decoder.decode(&stores, &mut bytes).unwrap_err();

        assert!(matches!(error, RedefmtDecoderError::InvalidStringBytes(_)))
    }

    #[test]
    fn tuple() {
        assert_value(TypeHint::Tuple, (10, "x"), |(num, str)| {
            ComplexValue::Tuple(vec![
                ComplexValue::Value(Value::U8(num)),
                ComplexValue::Value(Value::String(str.to_string())),
            ])
        });
        assert_value(TypeHint::Tuple, ((10, "x"), false), |((num, str), bool)| {
            let inner = ComplexValue::Tuple(vec![
                ComplexValue::Value(Value::U8(num)),
                ComplexValue::Value(Value::String(str.to_string())),
            ]);
            ComplexValue::Tuple(vec![inner, ComplexValue::Value(Value::Boolean(bool))])
        });
    }

    #[test]
    fn dyn_list() {
        assert_value(TypeHint::DynList, [&10u8 as &dyn WriteValue, &"x"], |_| {
            ComplexValue::List(vec![
                ComplexValue::Value(Value::U8(10)),
                ComplexValue::Value(Value::String("x".to_string())),
            ])
        });
    }

    #[test]
    fn list() {
        assert_value(TypeHint::List, [[0u8, 1], [2, 3]], |_| {
            ComplexValue::List(vec![
                ComplexValue::List(vec![
                    ComplexValue::Value(Value::U8(0)),
                    ComplexValue::Value(Value::U8(1)),
                ]),
                ComplexValue::List(vec![
                    ComplexValue::Value(Value::U8(2)),
                    ComplexValue::Value(Value::U8(3)),
                ]),
            ])
        });
    }

    #[test]
    fn write_segments() {
        // setup
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, stores) = Stores::mock(&cache);

        // input
        let combined_format_string = {
            let format_string = FormatString::parse("x = {}").unwrap();
            let provided_args = syn::parse_quote!(x = x);
            CombinedFormatString::combine(format_string, provided_args).unwrap()
        };

        let arg_value = true;

        let write_statement = WriteStatement::FormatString(combined_format_string.clone());

        // expected
        let expected_value = ComplexValue::NestedFormatString(
            &combined_format_string,
            vec![ComplexValue::Value(Value::Boolean(arg_value))],
        );

        // seed crate and write statement
        let write_id = {
            let crate_record = Crate::new(CrateName::new("x").unwrap());
            let crate_id = stores.main_db.insert(&crate_record).unwrap();
            let crate_context = stores.get_or_insert_crate(crate_id).unwrap();
            let write_statement_id = crate_context.db.insert(&write_statement).unwrap();

            (crate_id, write_statement_id)
        };

        // encode segment + arg
        let mut dispatcher = SimpleTestDispatcher::default();
        write_id.write_value(&mut dispatcher);
        arg_value.write_value(&mut dispatcher);

        assert_value_impl(stores, dispatcher, TypeHint::WriteId, expected_value);
    }

    fn assert_simple_value<T: WriteValue>(type_hint: TypeHint, encoded_value: T, from_inner: impl FnOnce(T) -> Value) {
        assert_value(type_hint, encoded_value, |t| ComplexValue::Value(from_inner(t)));
    }

    fn assert_value<'cache, T: WriteValue>(
        type_hint: TypeHint,
        encoded_value: T,
        from_inner: impl FnOnce(T) -> ComplexValue<'cache>,
    ) {
        let cache = RedefmtDecoderCache::default();
        let (_dir_guard, stores) = Stores::mock(&cache);

        let mut dispatcher = SimpleTestDispatcher::default();
        encoded_value.write_value(&mut dispatcher);

        let expected_value = from_inner(encoded_value);

        assert_value_impl(stores, dispatcher, type_hint, expected_value);
    }

    fn assert_value_impl<'a, 'cache>(
        stores: Stores<'a>,
        dispatcher: SimpleTestDispatcher,
        type_hint: TypeHint,
        expected_value: ComplexValue<'cache>,
    ) {
        let mut dispatched_bytes = dispatcher.bytes;

        let dispatched_type_hint = TypeHint::from_repr(dispatched_bytes.get_u8()).unwrap();

        assert_eq!(dispatched_type_hint, type_hint);

        let mut value_decoder = ValueDecoder::new(PointerWidth::of_target(), type_hint);

        let actual_value = value_decoder.decode(&stores, &mut dispatched_bytes).unwrap().unwrap();

        assert_eq!(expected_value, actual_value);
    }
}
