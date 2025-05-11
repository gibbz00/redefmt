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
        let mut dispatcher = SimpleTestDispatcher::default();

        true.write_value(&mut dispatcher);

        let mut expected_bytes = BytesMut::new();
        expected_bytes.put_u8(TypeHint::Boolean as u8);
        expected_bytes.put_u8(true as u8);

        assert_eq!(expected_bytes, dispatcher.bytes)
    }
}
