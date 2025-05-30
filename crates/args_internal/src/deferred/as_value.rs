use alloc::{string::String, vec::Vec};

use crate::*;

#[sealed::sealed]
pub trait AsDeferredValue {
    fn as_deferred_value(&self) -> DeferredValue;
}

#[sealed::sealed]
impl<T: AsDeferredValue> AsDeferredValue for &T {
    fn as_deferred_value(&self) -> DeferredValue {
        T::as_deferred_value(self)
    }
}

#[sealed::sealed]
impl AsDeferredValue for str {
    fn as_deferred_value(&self) -> DeferredValue {
        DeferredValue::String(self)
    }
}

#[sealed::sealed]
impl AsDeferredValue for String {
    fn as_deferred_value(&self) -> DeferredValue {
        DeferredValue::String(self)
    }
}

macro_rules! impl_copy {
    ($t:ty, $v:ident) => {
        #[sealed::sealed]
        impl AsDeferredValue for $t {
            fn as_deferred_value(&self) -> DeferredValue {
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
        fn as_deferred_value(&self) -> DeferredValue {
            let list = self.iter().map(|element| element.as_deferred_value()).collect();
            DeferredValue::List(list)
        }
    };
    ($ty:ty) => {
        #[sealed::sealed]
        impl AsDeferredValue for $ty {
            impl_list!();
        }
    };
    (T, $ty:ty) => {
        #[sealed::sealed]
        impl<T: AsDeferredValue> AsDeferredValue for $ty {
            impl_list!();
        }
    };
    (N, $ty:ty) => {
        #[sealed::sealed]
        impl<const N: usize> AsDeferredValue for $ty {
            impl_list!();
        }
    };
    (T, N, $ty:ty) => {
        #[sealed::sealed]
        impl<T: AsDeferredValue, const N: usize> AsDeferredValue for $ty {
            impl_list!();
        }
    };
}

impl_list!(T, &[T]);
impl_list!(T, N, [T; N]);
impl_list!(&[&dyn AsDeferredValue]);
impl_list!(N, [&dyn AsDeferredValue; N]);
impl_list!(T, Vec<T>);
impl_list!(Vec<&dyn AsDeferredValue>);

redefmt_internal_macros::impl_tuple_as_deferred_value!(7);
