use crate::*;

#[sealed::sealed]
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

// Can't do blanket implementation `impl<T: WriteValue> Format for T`
// as it will conflict with `impl<T: Format> Format for &T`, hence the manual
// impls.
macro_rules! body {
    () => {
        fn fmt(&self, f: &mut Formatter) {
            f.write(self);
        }
    };
}
macro_rules! impl_format {
    ($t:ty) => {
        impl Format for $t {
            body!();
        }
    };
    (T, $t:ty) => {
        impl<T: WriteValue> Format for $t {
            body!();
        }
    };
    (N, $t:ty) => {
        impl<const N: usize> Format for $t {
            body!();
        }
    };
    (T, N, $t:ty) => {
        impl<T: WriteValue, const N: usize> Format for $t {
            body!();
        }
    };
}

#[sealed::sealed]
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

#[sealed::sealed]
impl WriteValue for (CrateId, WriteStatementId) {
    fn hint(&self) -> TypeHint {
        TypeHint::WriteId
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        let (crate_id, statement_id) = self;
        crate_id.as_ref().write_raw(dispatcher);
        statement_id.as_ref().write_raw(dispatcher);
    }
}

// NOTE:
// Increasing this from 7 to 10 increases crate build time by almost three fold.
// From ~2.4s to ~7.1s on the authors computer. Codegen duration remains the
// same at about 0.1s. Not pulling in this proc macro at all removes about 0.2s
// from cold compile times. Simply moving this to a build script is therefore a
// slightly premature optimization given that it is generated code itself which
// takes the longest to compile.
redefmt_internal_macros::impl_tuple_write_value!(7);

macro_rules! num_impl {
    ($(($type:ty, $hint:expr),)*) => {
        $(
            #[::sealed::sealed]
            impl WriteValue for $type {
                fn hint(&self) -> TypeHint {
                    $hint
                }

                fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
                    dispatcher.write(&self.to_be_bytes());
                }
            }

            impl_format!($type);
        )*
    };
}

num_impl!(
    (isize, TypeHint::Isize),
    (i8, TypeHint::I8),
    (i16, TypeHint::I16),
    (i32, TypeHint::I32),
    (i64, TypeHint::I64),
    (i128, TypeHint::I128),
    (usize, TypeHint::Usize),
    (u8, TypeHint::U8),
    (u16, TypeHint::U16),
    (u32, TypeHint::U32),
    (u64, TypeHint::U64),
    (u128, TypeHint::U128),
    (f32, TypeHint::F32),
    (f64, TypeHint::F64),
);

impl_format!(bool);
#[sealed::sealed]
impl WriteValue for bool {
    fn hint(&self) -> TypeHint {
        TypeHint::Boolean
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        dispatcher.write(&(*self as u8).to_be_bytes());
    }
}

impl_format!(char);
#[sealed::sealed]
impl WriteValue for char {
    fn hint(&self) -> TypeHint {
        TypeHint::Char
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        let mut byte_buf = [0; 4];
        let char_str = self.encode_utf8(&mut byte_buf);

        dispatcher.write(&[char_str.len() as u8]);
        dispatcher.write(char_str.as_bytes());
    }
}

impl_format!(&str);
#[sealed::sealed]
impl WriteValue for &str {
    fn hint(&self) -> TypeHint {
        TypeHint::StringSlice
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        dispatcher.write(&self.len().to_be_bytes());
        dispatcher.write(self.as_bytes());
    }
}

impl_format!(T, &[T]);
#[sealed::sealed]
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

impl_format!(T, N, [T; N]);
#[sealed::sealed]
impl<T: WriteValue, const N: usize> WriteValue for [T; N] {
    fn hint(&self) -> TypeHint {
        TypeHint::List
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        self.as_slice().write_raw(dispatcher);
    }
}

impl_format!(&[&dyn WriteValue]);
#[sealed::sealed]
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

impl_format!(N, [&dyn WriteValue; N]);
#[sealed::sealed]
impl<const N: usize> WriteValue for [&dyn WriteValue; N] {
    fn hint(&self) -> TypeHint {
        TypeHint::DynList
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        self.as_slice().write_raw(dispatcher);
    }
}

#[cfg(feature = "alloc")]
impl_format!(T, alloc::vec::Vec<T>);
#[cfg(feature = "alloc")]
#[sealed::sealed]
impl<T: WriteValue> WriteValue for alloc::vec::Vec<T> {
    fn hint(&self) -> TypeHint {
        TypeHint::List
    }

    fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
        self.as_slice().write_raw(dispatcher);
    }
}

#[cfg(feature = "alloc")]
impl_format!(alloc::vec::Vec<&dyn WriteValue>);
#[cfg(feature = "alloc")]
#[sealed::sealed]
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
    use alloc::{boxed::Box, vec::Vec};

    use bytes::{BufMut, BytesMut};

    use super::*;

    // NOTE: Most of these tests don't necessarily verify that the resulting
    // bytes have an "intended value", but act more assertions that different
    // type compinations still compile for situations where `format_args!` would.
    //
    // True tests that the encoded bytes are indeed correct is instead done in
    // the decoder tests.

    #[test]
    fn write_value() {
        let crate_id = CrateId::new(1);
        let statement_id = WriteStatementId::new(2);

        let mut dispatcher = SimpleTestDispatcher::default();
        (crate_id, statement_id).write_value(&mut dispatcher);

        let mut expected_bytes = alloc::vec![TypeHint::WriteId as u8];
        expected_bytes.extend_from_slice(&crate_id.as_ref().to_be_bytes());
        expected_bytes.extend_from_slice(&statement_id.as_ref().to_be_bytes());

        assert_eq!(expected_bytes.as_slice(), dispatcher.bytes,);
    }

    #[test]
    fn num() {
        assert_num::<isize>(TypeHint::Isize);
        assert_num::<i8>(TypeHint::I8);
        assert_num::<i16>(TypeHint::I16);
        assert_num::<i32>(TypeHint::I32);
        assert_num::<i64>(TypeHint::I64);
        assert_num::<i128>(TypeHint::I128);
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
    fn char() {
        assert_char('x', 1);
        assert_char('ß', 2);
        assert_char('ᴪ', 3);
        assert_char('🦀', 4);

        fn assert_char(char: char, expected_length: u8) {
            let mut dispatcher = SimpleTestDispatcher::default();
            char.write_value(&mut dispatcher);

            let mut expected_bytes = alloc::vec![TypeHint::Char as u8, expected_length];

            {
                let mut str_buf: [u8; 4] = [0; 4];
                let str: &str = char.encode_utf8(&mut str_buf);
                assert_eq!(expected_length, str.len() as u8);

                expected_bytes.extend_from_slice(str.as_bytes());
            }

            let expected_bytes_length = 2 + expected_length;
            assert_eq!(expected_bytes_length, dispatcher.bytes.len() as u8);

            assert_eq!(expected_bytes, dispatcher.bytes);
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
    fn boxed() {
        assert_boxed("");
        assert_boxed(123);

        fn assert_boxed(value: impl WriteValue) {
            let mut boxed_dispatcher = SimpleTestDispatcher::default();
            let boxed = Box::new(&value);
            boxed.write_value(&mut boxed_dispatcher);

            let mut unboxed_dispatcher = SimpleTestDispatcher::default();
            value.write_value(&mut unboxed_dispatcher);

            assert_eq!(unboxed_dispatcher.bytes, boxed_dispatcher.bytes)
        }
    }

    #[test]
    fn boxed_dyn() {
        let value = "123";

        let mut boxed_dispatcher = SimpleTestDispatcher::default();
        let boxed_value: Box<dyn WriteValue> = Box::new(value);
        boxed_value.write_value(&mut boxed_dispatcher);

        let mut unboxed_dispatcher = SimpleTestDispatcher::default();
        value.write_value(&mut unboxed_dispatcher);

        assert_eq!(unboxed_dispatcher.bytes, boxed_dispatcher.bytes)
    }

    #[test]
    fn tuple() {
        let mut dispatcher = SimpleTestDispatcher::default();
        ("a", 1).write_value(&mut dispatcher);

        let mut expected_dispatcher = SimpleTestDispatcher::default();
        expected_dispatcher.bytes.extend_from_slice(&[TypeHint::Tuple as u8, 2]);
        "a".write_value(&mut expected_dispatcher);
        1.write_value(&mut expected_dispatcher);

        assert_eq!(expected_dispatcher.bytes, dispatcher.bytes,)
    }

    #[test]
    fn compiles_tuple_permutations() {
        let mut dispatcher = NoopTestDispatcher;

        // All permutations for length = 2
        ("str", 10).write_value(&mut dispatcher);
        (&"str" as &dyn WriteValue, 10).write_value(&mut dispatcher);
        ("str", &10 as &dyn WriteValue).write_value(&mut dispatcher);
        (&"str" as &dyn WriteValue, &10 as &dyn WriteValue).write_value(&mut dispatcher);

        // Some permutation for length > 2
        (&"str" as &dyn WriteValue, &10 as &dyn WriteValue, 5, false).write_value(&mut dispatcher);

        // Write value for max length = 7
        (0, 1, 2, 3, 4, 5, 6).write_value(&mut dispatcher);
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
