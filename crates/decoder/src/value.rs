#[derive(Debug)]
pub enum Value {
    Primitive(Primitive),
    Collection(Collection),
    Type(Type),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Collection {
    String(String),
    Slice(Vec<Value>),
    Tuple(Vec<Value>),
    Set(Vec<Value>),
    Map(Vec<(Value, Value)>),
}

#[derive(Debug)]
pub struct Type {
    name: String,
    variant: TypeVariant,
}

#[derive(Debug)]
pub enum TypeVariant {
    Struct(StructVariant),
    Enum(Vec<(String, StructVariant)>),
}

#[derive(Debug)]
pub enum StructVariant {
    Unit,
    Tuple(Vec<Value>),
    Named(Vec<(String, Value)>),
}
