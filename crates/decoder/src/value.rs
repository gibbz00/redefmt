use redefmt_db::statement_table::stored_format_expression::StoredFormatExpression;

#[derive(Debug, PartialEq)]
pub enum Value {
    // Primitives
    Boolean(bool),
    // FIXME: just use usize and return error if it does not fit
    Usize(u64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    // FIXME: just use isize and return error if it does not fit
    Isize(i64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
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
    Type(TypeStructureValue<'cache>),
    NestedFormatExpression(&'cache StoredFormatExpression<'static>, Vec<ComplexValue<'cache>>),
    // TODO:
    // Set(Vec<Value>),
    // Map(Vec<(Value, Value)>),
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
    Tuple(Vec<ComplexValue<'cache>>),
    Named(Vec<(&'cache str, ComplexValue<'cache>)>),
}
