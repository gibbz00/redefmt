use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::*;

pub trait AsDeferredValue: private::Sealed {
    fn as_deferred_value(&self) -> DeferredValue<'_>;
}

mod private {
    pub trait Sealed {}
}

macro_rules! impl_sealed {
    ($type:ty) => {
        impl private::Sealed for $type {}
    };
    (T, $type:ty) => {
        impl<T: AsDeferredValue> private::Sealed for $type {}
    };
    (N, $t:ty) => {
        impl<const N: usize> private::Sealed for $t {}
    };
    (T, N, $t:ty) => {
        impl<T: AsDeferredValue, const N: usize> private::Sealed for $t {}
    };
}

impl_sealed!(T, &T);
impl<T: AsDeferredValue> AsDeferredValue for &T {
    fn as_deferred_value(&self) -> DeferredValue<'_> {
        T::as_deferred_value(self)
    }
}

impl_sealed!(DeferredValue<'_>);
impl AsDeferredValue for DeferredValue<'_> {
    fn as_deferred_value(&self) -> DeferredValue<'_> {
        self.clone()
    }
}

impl_sealed!(&str);
impl AsDeferredValue for &str {
    fn as_deferred_value(&self) -> DeferredValue<'_> {
        DeferredValue::String(Cow::Borrowed(self))
    }
}

impl_sealed!(String);
impl AsDeferredValue for String {
    fn as_deferred_value(&self) -> DeferredValue<'_> {
        DeferredValue::String(Cow::Borrowed(self))
    }
}

macro_rules! impl_copy {
    ($t:ty, $v:ident) => {
        impl_sealed!($t);
        impl AsDeferredValue for $t {
            fn as_deferred_value(&self) -> DeferredValue<'_> {
                DeferredValue::$v(*self)
            }
        }
    };
}

impl_copy!(bool, Boolean);
impl_copy!(usize, Usize);
impl_copy!(u8, U8);
impl_copy!(u16, U16);
impl_copy!(u32, U32);
impl_copy!(u64, U64);
impl_copy!(u128, U128);
impl_copy!(isize, Isize);
impl_copy!(i8, I8);
impl_copy!(i16, I16);
impl_copy!(i32, I32);
impl_copy!(i64, I64);
impl_copy!(i128, I128);
impl_copy!(f32, F32);
impl_copy!(f64, F64);
impl_copy!(char, Char);

macro_rules! impl_list {
    () => {
        fn as_deferred_value(&self) -> DeferredValue<'_> {
            let list = self.iter().map(|element| element.as_deferred_value()).collect();
            DeferredValue::List(list)
        }
    };
    ($ty:ty) => {
        impl AsDeferredValue for $ty {
            impl_list!();
        }
    };
    (T, $ty:ty) => {
        impl<T: AsDeferredValue> AsDeferredValue for $ty {
            impl_list!();
        }
    };
    (N, $ty:ty) => {
        impl<const N: usize> AsDeferredValue for $ty {
            impl_list!();
        }
    };
    (T, N, $ty:ty) => {
        impl<T: AsDeferredValue, const N: usize> AsDeferredValue for $ty {
            impl_list!();
        }
    };
}

macro_rules! impl_list_combined {
    ($($tt:tt)*) => {
        impl_sealed!($($tt)*);
        impl_list!($($tt)*);
    };
}

impl_list_combined!(T, &[T]);
impl_list_combined!(T, N, [T; N]);
impl_list_combined!(&[&dyn AsDeferredValue]);
impl_list_combined!(N, [&dyn AsDeferredValue; N]);
impl_list_combined!(T, Vec<T>);
impl_list_combined!(Vec<&dyn AsDeferredValue>);

redefmt_utils_tupler::impl_tuple_as_deferred_value!(7);
