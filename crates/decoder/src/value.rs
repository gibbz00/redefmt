use redefmt_args::provided_args::CombinedFormatString;

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
    Char(char),
    String(String),
}

#[derive(Debug, PartialEq)]
pub enum ComplexValue<'cache> {
    Value(Value),
    // Reused for array, vec and slice containing both single and dyn values.
    List(Vec<ComplexValue<'cache>>),
    Tuple(Vec<ComplexValue<'cache>>),
    Type(Type<'cache>),
    NestedFormatString(&'cache CombinedFormatString<'static>, Vec<ComplexValue<'cache>>),
    // TODO:
    // Set(Vec<Value>),
    // Map(Vec<(Value, Value)>),
}

#[derive(Debug, PartialEq)]
pub struct Type<'cache> {
    name: String,
    variant: TypeVariant<'cache>,
}

#[derive(Debug, PartialEq)]
pub enum TypeVariant<'cache> {
    Struct(StructVariant<'cache>),
    Enum(Vec<(String, StructVariant<'cache>)>),
}

#[derive(Debug, PartialEq)]
pub enum StructVariant<'cache> {
    Unit,
    Tuple(Vec<ComplexValue<'cache>>),
    Named(Vec<(String, ComplexValue<'cache>)>),
}
