//! # `redefmt-decoder`

// TEMP:
#![allow(missing_docs)]

mod core;
pub use core::{RedefmtDecoder, RedefmtDecoderError};

mod frame {
    use redefmt_args::FormatOptions;
    use redefmt_common::codec::Stamp;

    use crate::*;

    pub struct RedefmtFrame {
        stamp: Option<Stamp>,
        segments: Vec<Segment>,
    }

    pub enum Segment {
        Literal(String),
        Value(Value, FormatOptions<'static>),
    }
}
pub(crate) use frame::{RedefmtFrame, Segment};

mod value {
    pub enum Value {
        Primitive(Primitive),
        Collection(Collection),
        Type(Type),
    }

    pub enum Primitive {
        Boolean(bool),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        U128(u128),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        F32(f32),
        F64(f64),
    }

    pub enum Collection {
        String(String),
        Slice(Vec<Value>),
        Tuple(Vec<Value>),
        Set(Vec<Value>),
        Map(Vec<(Value, Value)>),
    }

    pub struct Type {
        name: String,
        variant: TypeVariant,
    }

    pub enum TypeVariant {
        Struct(StructVariant),
        Enum(Vec<(String, StructVariant)>),
    }

    pub enum StructVariant {
        Unit,
        Tuple(Vec<Value>),
        Named(Vec<(String, Value)>),
    }
}
pub(crate) use value::Value;
