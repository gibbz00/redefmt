#[derive(Debug, PartialEq)]
pub enum Value {
    // Primitives
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
    F32(f32),
    F64(f64),

    // Collections
    String(String),
    Slice(Vec<Value>),
    Tuple(Vec<Value>),
    Set(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Type(Type),
}

#[derive(Debug, PartialEq)]
pub struct Type {
    name: String,
    variant: TypeVariant,
}

#[derive(Debug, PartialEq)]
pub enum TypeVariant {
    Struct(StructVariant),
    Enum(Vec<(String, StructVariant)>),
}

#[derive(Debug, PartialEq)]
pub enum StructVariant {
    Unit,
    Tuple(Vec<Value>),
    Named(Vec<(String, Value)>),
}
