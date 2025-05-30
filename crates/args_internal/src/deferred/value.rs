use alloc::{string::String, vec::Vec};

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum DeferredValue<'a> {
    Boolean(bool),
    // Does not contain a usize since target usize
    // may be greater than host usize.
    Usize(u64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    // Does not contain a isize since target isize
    // may be greater than host isize.
    Isize(i64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),
    Char(char),
    String(&'a str),
    // Reused for array, vec and slice
    List(Vec<DeferredValue<'a>>),
    Tuple(Vec<DeferredValue<'a>>),
    Type(TypeValue<'a>),
}

impl<'a> DeferredValue<'a> {
    pub fn evaluate(
        &self,
        _string_buffer: &mut String,
        _format_options: &FormatOptions,
        _provided_args: &DeferredProvidedArgs,
    ) -> String {
        _string_buffer.push('a');
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeValue<'a> {
    pub name: &'a str,
    pub variant: TypeVariant<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeVariant<'a> {
    Struct(StructVariant<'a>),
    Enum((&'a str, StructVariant<'a>)),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructVariant<'a> {
    Unit,
    Tuple(&'a [DeferredValue<'a>]),
    Named(&'a [(&'a str, DeferredValue<'a>)]),
}
