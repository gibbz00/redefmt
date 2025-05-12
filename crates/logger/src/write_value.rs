use redefmt_common::codec::TypeHint;

use crate::*;

// TODO: seal
pub trait WriteValue {
    fn write_value(&self, dispatcher: &mut dyn Dispatcher);
}

macro_rules! num_impl {
    ($(($type:ty, $hint:expr),)*) => {
        $(
            impl WriteValue for $type {
                fn write_value(&self, dispatcher: &mut dyn Dispatcher) {
                    dispatcher.write(&($hint as u8).to_be_bytes());
                    dispatcher.write(&self.to_be_bytes());
                }
            }
        )*
    };
}

num_impl!(
    (isize, TypeHint::Isize),
    (i8, TypeHint::I8),
    (i16, TypeHint::I16),
    (i32, TypeHint::I32),
    (i64, TypeHint::I64),
    (usize, TypeHint::Usize),
    (u8, TypeHint::U8),
    (u16, TypeHint::U16),
    (u32, TypeHint::U32),
    (u64, TypeHint::U64),
    (u128, TypeHint::U128),
    (f32, TypeHint::F32),
    (f64, TypeHint::F64),
);

impl WriteValue for bool {
    fn write_value(&self, dispatcher: &mut dyn Dispatcher) {
        dispatcher.write(&(TypeHint::Boolean as u8).to_be_bytes());
        dispatcher.write(&(*self as u8).to_be_bytes());
    }
}

// FIXME: impls must write type hint

impl WriteValue for &[u8] {
    fn write_value(&self, dispatcher: &mut dyn Dispatcher) {
        self.len().write_value(dispatcher);
        dispatcher.write(self);
    }
}

impl WriteValue for &str {
    fn write_value(&self, dispatcher: &mut dyn Dispatcher) {
        self.len().write_value(dispatcher);
        dispatcher.write(self.as_bytes());
    }
}

impl<const N: usize> WriteValue for [u8; N] {
    fn write_value(&self, dispatcher: &mut dyn Dispatcher) {
        // No need to send length, should have been registered in write!/log! statement
        dispatcher.write(self);
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn bool() {
        assert_bool(true);
        assert_bool(false);

        fn assert_bool(boolean: bool) {
            let mut dispatcher = SimpleTestDispatcher::default();

            boolean.write_value(&mut dispatcher);

            let mut expected_bytes = BytesMut::new();
            expected_bytes.put_u8(TypeHint::Boolean as u8);
            expected_bytes.put_u8(boolean as u8);

            assert_eq!(expected_bytes, dispatcher.bytes, "invalid bytes for {boolean}")
        }
    }

    #[test]
    fn num() {
        assert_num::<isize>(TypeHint::Isize);
        assert_num::<i8>(TypeHint::I8);
        assert_num::<i16>(TypeHint::I16);
        assert_num::<i32>(TypeHint::I32);
        assert_num::<i64>(TypeHint::I64);
        assert_num::<usize>(TypeHint::Usize);
        assert_num::<u8>(TypeHint::U8);
        assert_num::<u16>(TypeHint::U16);
        assert_num::<u32>(TypeHint::U32);
        assert_num::<u64>(TypeHint::U64);
        assert_num::<u128>(TypeHint::U128);
        assert_num::<f32>(TypeHint::F32);
        assert_num::<f64>(TypeHint::F64);

        fn assert_num<T: WriteValue + num_traits::ToBytes + num_traits::One>(type_hint: TypeHint) {
            let mut dispatcher = SimpleTestDispatcher::default();

            let num = T::one();

            num.write_value(&mut dispatcher);

            let mut expected_bytes = BytesMut::new();
            expected_bytes.put_u8(type_hint as u8);
            expected_bytes.put_slice(num.to_be_bytes().as_ref());

            assert_eq!(expected_bytes, dispatcher.bytes, "invalid bytes for {type_hint}")
        }
    }
}
