use alloc::{string::String, vec::Vec};

use crate::*;

#[derive(Debug, PartialEq)]
pub enum DeferredFormatValue<'a> {
    Boolean(bool),
    Usize(u64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
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
    List(Vec<DeferredFormatValue<'a>>),
    Tuple(Vec<DeferredFormatValue<'a>>),
    Type(TypeValue<'a>),
}

impl<'a> DeferredFormatValue<'a> {
    pub fn evaluate(
        &self,
        string_buffer: &mut String,
        format_options: &FormatOptions,
        provided_args: &DeferredProvidedArgs,
    ) -> String {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct TypeValue<'a> {
    pub name: &'a str,
    pub variant: TypeVariant<'a>,
}

#[derive(Debug, PartialEq)]
pub enum TypeVariant<'a> {
    Struct(StructVariant<'a>),
    Enum((&'a str, StructVariant<'a>)),
}

#[derive(Debug, PartialEq)]
pub enum StructVariant<'a> {
    Unit,
    Tuple(Vec<DeferredFormatValue<'a>>),
    Named(Vec<(&'a str, DeferredFormatValue<'a>)>),
}
