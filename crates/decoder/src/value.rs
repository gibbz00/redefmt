use redefmt_args::deferred::DeferredFormatExpression;

#[derive(Debug, PartialEq)]
pub enum Value<'cache> {
    Boolean(bool),
    // u64 in case host usize < target usize
    Usize(u64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    // i64 in case host isize < target isize
    Isize(i64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),
    Char(char),
    String(String),
    // Reused for array, vec and slice containing both single and dyn values.
    List(Vec<Value<'cache>>),
    Tuple(Vec<Value<'cache>>),
    Type(TypeStructureValue<'cache>),
    NestedFormatExpression {
        expression: &'cache DeferredFormatExpression<'static>,
        append_newline: bool,
        decoded_values: Vec<Value<'cache>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct TypeStructureValue<'cache> {
    pub name: &'cache str,
    pub variant: TypeStructureVariantValue<'cache>,
}

#[derive(Debug, PartialEq)]
pub enum TypeStructureVariantValue<'cache> {
    Struct(StructVariantValue<'cache>),
    Enum((&'cache str, StructVariantValue<'cache>)),
}

#[derive(Debug, PartialEq)]
pub enum StructVariantValue<'cache> {
    Unit,
    Tuple(Vec<Value<'cache>>),
    Named(Vec<(&'cache str, Value<'cache>)>),
}
