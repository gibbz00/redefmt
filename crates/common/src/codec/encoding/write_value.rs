use crate::*;

// TODO: seal
pub trait WriteValue {
    // Can't make associated const as `WriteValue` needs to be dyn compatible,
    // and const fn in traits does not yet exist.
    fn hint(&self) -> TypeHint;

    fn write_value(&self, dispatcher: &mut dyn Dispatcher) {
        write_type_hint(self.hint(), dispatcher);
        self.write_raw(dispatcher);
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher);
}

impl<T: WriteValue> WriteValue for &T {
    fn hint(&self) -> TypeHint {
        T::hint(self)
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        T::write_raw(self, dispatcher)
    }
}

fn write_type_hint(type_hint: TypeHint, dispatcher: &mut dyn Dispatcher) {
    dispatcher.write(&(type_hint as u8).to_be_bytes());
}

macro_rules! num_impl {
    ($(($type:ty, $hint:expr),)*) => {
        $(
            impl WriteValue for $type {
                fn hint(&self) -> TypeHint {
                    $hint
                }

                fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
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
    fn hint(&self) -> TypeHint {
        TypeHint::Boolean
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        dispatcher.write(&(*self as u8).to_be_bytes());
    }
}

impl WriteValue for &str {
    fn hint(&self) -> TypeHint {
        TypeHint::StringSlice
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        dispatcher.write(&self.len().to_be_bytes());
        dispatcher.write(self.as_bytes());
    }
}

impl<T: WriteValue> WriteValue for &[T] {
    fn hint(&self) -> TypeHint {
        TypeHint::List
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        dispatcher.write(&self.len().to_be_bytes());

        // NOTE:
        // Type hint need not be repeated for each element,
        // and none is written if length is zero.
        for (index, element) in self.iter().enumerate() {
            if index == 0 {
                write_type_hint(element.hint(), dispatcher);
            }

            element.write_raw(dispatcher);
        }
    }
}

impl<T: WriteValue, const N: usize> WriteValue for [T; N] {
    fn hint(&self) -> TypeHint {
        TypeHint::List
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        self.as_slice().write_raw(dispatcher);
    }
}

impl WriteValue for &[&dyn WriteValue] {
    fn hint(&self) -> TypeHint {
        TypeHint::DynList
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        dispatcher.write(&self.len().to_be_bytes());

        for element in *self {
            // type hint needs to be written for each element
            element.write_value(dispatcher);
        }
    }
}

impl<const N: usize> WriteValue for [&dyn WriteValue; N] {
    fn hint(&self) -> TypeHint {
        TypeHint::DynList
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        self.as_slice().write_raw(dispatcher);
    }
}

#[cfg(feature = "alloc")]
impl<T: WriteValue> WriteValue for alloc::vec::Vec<T> {
    fn hint(&self) -> TypeHint {
        TypeHint::List
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        self.as_slice().write_raw(dispatcher);
    }
}

#[cfg(feature = "alloc")]
impl WriteValue for alloc::vec::Vec<&dyn WriteValue> {
    fn hint(&self) -> TypeHint {
        TypeHint::DynList
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        self.as_slice().write_raw(dispatcher);
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use bytes::{BufMut, BytesMut};

    use super::*;

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
    fn str() {
        assert_str("x");
        assert_str("🦀");

        fn assert_str(str: &str) {
            let mut dispatcher = SimpleTestDispatcher::default();

            str.write_value(&mut dispatcher);

            let mut expected_bytes = BytesMut::new();
            expected_bytes.put_u8(TypeHint::StringSlice as u8);
            expected_bytes.put_slice(&str.len().to_be_bytes());
            expected_bytes.put_slice(str.as_bytes());

            assert_eq!(expected_bytes, dispatcher.bytes, "invalid bytes for {str}")
        }
    }

    #[test]
    fn slice() {
        assert_slice::<u8>(&[]);
        let array = [0];
        assert_slice(&array);
        assert_slice(array.as_ref());
        assert_slice(&[0, 1]);
        assert_slice(&["a", "b", "c"]);
        assert_slice(&[&[0], &[1]]);
        assert_slice(&[[1].as_slice(), &[2, 3], &[4, 5, 6]]);
    }

    #[test]
    fn array() {
        assert_array::<u8, 0>([]);
        assert_array([0, 1]);
        assert_array([[0], [1]]);
        assert_array([&[0], &[1]]);
    }

    #[test]
    fn vec() {
        assert_vec::<u8>(alloc::vec![]);
        assert_vec(alloc::vec![0, 1]);
        assert_vec(alloc::vec![["y"], ["y"]]);
    }

    #[test]
    fn dyn_slice() {
        assert_dyn_slice(&[]);
        assert_dyn_slice(&[&"str" as &dyn WriteValue, &1 as &dyn WriteValue]);
    }

    #[test]
    fn dyn_array() {
        assert_dyn_array([]);
        assert_dyn_array([&"str" as &dyn WriteValue, &1 as &dyn WriteValue]);
    }

    #[test]
    fn dyn_vec() {
        assert_dyn_vec(alloc::vec![]);
        assert_dyn_vec(alloc::vec![&"str" as &dyn WriteValue, &1 as &dyn WriteValue]);
    }

    fn assert_array<T: WriteValue + core::fmt::Debug, const N: usize>(array: [T; N]) {
        let mut dispatcher = SimpleTestDispatcher::default();
        array.write_value(&mut dispatcher);

        let expected_bytes = expected_slice_bytes(&array);
        assert_eq!(expected_bytes, dispatcher.bytes, "invalid bytes for {array:?}")
    }

    fn assert_slice<T: WriteValue + core::fmt::Debug>(slice: &[T]) {
        let mut dispatcher = SimpleTestDispatcher::default();
        slice.write_value(&mut dispatcher);

        let expected_bytes = expected_slice_bytes(slice);
        assert_eq!(expected_bytes, dispatcher.bytes, "invalid bytes for {slice:?}")
    }

    fn assert_vec<T: WriteValue + core::fmt::Debug>(vec: Vec<T>) {
        let mut dispatcher = SimpleTestDispatcher::default();
        vec.write_value(&mut dispatcher);
        let expected_bytes = expected_slice_bytes(&vec);
        assert_eq!(expected_bytes, dispatcher.bytes, "invalid bytes for {vec:?}")
    }

    fn expected_slice_bytes<T: WriteValue>(slice: &[T]) -> BytesMut {
        let mut expected_bytes = BytesMut::new();
        expected_bytes.put_u8(TypeHint::List as u8);
        expected_bytes.put_slice(&slice.len().to_be_bytes());

        for (index, element) in slice.iter().enumerate() {
            if index == 0 {
                expected_bytes.put_u8(element.hint() as u8);
            }

            let mut expected_dispatcher = SimpleTestDispatcher::default();
            element.write_raw(&mut expected_dispatcher);
            expected_bytes.put_slice(&expected_dispatcher.bytes);
        }

        expected_bytes
    }

    fn assert_dyn_slice(dyn_slice: &[&dyn WriteValue]) {
        let mut dispatcher = SimpleTestDispatcher::default();
        dyn_slice.write_value(&mut dispatcher);

        let expected_bytes = expected_dyn_bytes(dyn_slice);
        assert_eq!(expected_bytes, dispatcher.bytes)
    }

    fn assert_dyn_array<const N: usize>(dyn_array: [&dyn WriteValue; N]) {
        let mut dispatcher = SimpleTestDispatcher::default();
        dyn_array.write_value(&mut dispatcher);

        let expected_bytes = expected_dyn_bytes(&dyn_array);
        assert_eq!(expected_bytes, dispatcher.bytes)
    }

    fn assert_dyn_vec(dyn_vec: Vec<&dyn WriteValue>) {
        let mut dispatcher = SimpleTestDispatcher::default();
        dyn_vec.write_value(&mut dispatcher);

        let expected_bytes = expected_dyn_bytes(&dyn_vec);
        assert_eq!(expected_bytes, dispatcher.bytes)
    }

    fn expected_dyn_bytes(slice: &[&dyn WriteValue]) -> BytesMut {
        let mut expected_bytes = BytesMut::new();
        expected_bytes.put_u8(TypeHint::DynList as u8);
        expected_bytes.put_slice(&slice.len().to_be_bytes());

        for element in slice {
            expected_bytes.put_u8(element.hint() as u8);

            let mut expected_dispatcher = SimpleTestDispatcher::default();
            element.write_raw(&mut expected_dispatcher);
            expected_bytes.put_slice(&expected_dispatcher.bytes);
        }

        expected_bytes
    }
}
