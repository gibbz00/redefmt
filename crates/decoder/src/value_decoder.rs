use encode_unicode::CharExt;
use redefmt_common::{
    codec::frame::{PointerWidth, TypeHint},
    identifiers::CrateId,
};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

use crate::*;

pub struct ValueDecoder<'caches> {
    pointer_width: PointerWidth,
    type_hint: TypeHint,
    length_context: Option<usize>,
    list_decoder: Option<ListValueDecoder<'caches>>,
    write_decoder: Option<WriteStatementDecoder<'caches>>,
}

impl<'caches> ValueDecoder<'caches> {
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
        stores: &DecoderStores<'caches>,
        src: &mut BytesMut,
    ) -> Result<Option<Value>, RedefmtDecoderError> {
        let maybe_value = match self.type_hint {
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

                let Some(values) = list_decoder.decode_dyn_list(stores, src)? else {
                    return Ok(None);
                };

                Some(Value::Tuple(values))
            }
            TypeHint::DynList => {
                let Some(list_decoder) = self.get_or_store_usize_list(src)? else {
                    return Ok(None);
                };

                let Some(values) = list_decoder.decode_dyn_list(stores, src)? else {
                    return Ok(None);
                };

                Some(Value::List(values))
            }
            TypeHint::List => {
                let Some(list_decoder) = self.get_or_store_usize_list(src)? else {
                    return Ok(None);
                };

                let Some(values) = list_decoder.decode_list(stores, src)? else {
                    return Ok(None);
                };

                Some(Value::List(values))
            }
            TypeHint::WriteId => {
                let Some(write_statement_decoder) = self.get_or_store_write_decoder(stores, src)? else {
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

    fn get_or_store_u8_list(&mut self, src: &mut BytesMut) -> Option<&mut ListValueDecoder<'caches>> {
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
    ) -> Result<Option<&mut ListValueDecoder<'caches>>, RedefmtDecoderError> {
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
        stores: &DecoderStores<'caches>,
        src: &mut BytesMut,
    ) -> Result<Option<&mut WriteStatementDecoder<'caches>>, RedefmtDecoderError> {
        if self.write_decoder.is_none() {
            let Ok(crate_id) = src.try_get_u16().map(CrateId::new) else {
                return Ok(None);
            };

            let write_crate = stores.get_or_insert_crate(crate_id)?;

            self.write_decoder = Some(WriteStatementDecoder { write_crate })
        }

        Ok(self.write_decoder.as_mut())
    }
}

#[cfg(test)]
mod tests {
    use redefmt_common::codec::encoding::{SimpleTestDispatcher, WriteValue};

    use super::*;

    #[test]
    fn boolean_false() {
        assert_value(TypeHint::Boolean, true, Value::Boolean);
        assert_value(TypeHint::Boolean, false, Value::Boolean);
    }

    #[test]
    fn boolean_err() {
        let cache = DecoderCache::default();
        let (_dir_guard, stores) = DecoderStores::mock(&cache);

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
        let cache = DecoderCache::default();
        let (_dir_guard, stores) = DecoderStores::mock(&cache);

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
        assert_value(TypeHint::StringSlice, "abc", |str| Value::String(str.to_string()));
        assert_value(TypeHint::StringSlice, "🦀", |str| Value::String(str.to_string()));
    }

    #[test]
    fn string_invalid_utf8_error() {
        let cache = DecoderCache::default();
        let (_dir_guard, stores) = DecoderStores::mock(&cache);

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

    fn assert_value<T: WriteValue>(type_hint: TypeHint, encoded_value: T, from_inner: fn(T) -> Value) {
        let cache = DecoderCache::default();
        let (_dir_guard, stores) = DecoderStores::mock(&cache);

        let mut dispatcher = SimpleTestDispatcher::default();
        encoded_value.write_value(&mut dispatcher);

        let mut dispatched_bytes = dispatcher.bytes;

        let dispatched_type_hint = TypeHint::from_repr(dispatched_bytes.get_u8()).unwrap();

        assert_eq!(dispatched_type_hint, type_hint);

        let mut value_decoder = ValueDecoder::new(PointerWidth::of_target(), type_hint);

        let actual_value = value_decoder.decode(&stores, &mut dispatched_bytes).unwrap().unwrap();

        let expected_value = from_inner(encoded_value);

        assert_eq!(expected_value, actual_value);
    }
}
