use redefmt_args::{identifier::AnyIdentifier, processor::ProcessedFormatString};
use redefmt_db::statement_table::stored_format_expression::StoredFormatExpression;

#[derive(Debug, Default, PartialEq)]
pub struct DecodedValues<'cache> {
    pub positional: Vec<Value<'cache>>,
    pub named: Vec<(&'cache AnyIdentifier<'static>, Value<'cache>)>,
}

impl<'cache> DecodedValues<'cache> {
    pub(crate) fn new_with_capacity(stored_expression: &StoredFormatExpression) -> Self {
        Self {
            positional: Vec::with_capacity(stored_expression.expected_positional_arg_count),
            named: Vec::with_capacity(stored_expression.expected_named_args.len()),
        }
    }

    pub(crate) fn is_filled(&self, stored_expression: &StoredFormatExpression) -> bool {
        self.positional.len() == stored_expression.expected_positional_arg_count
            && self.named.len() == stored_expression.expected_named_args.len()
    }

    pub(crate) fn push(
        &mut self,
        decoded_value: Value<'cache>,
        stored_expression: &'cache StoredFormatExpression<'static>,
    ) {
        if self.positional.len() < stored_expression.expected_positional_arg_count {
            self.positional.push(decoded_value);
        } else {
            let next_identifier_index = self.named.len();
            let identifier = &stored_expression.expected_named_args[next_identifier_index];
            self.named.push((identifier, decoded_value));
        }
    }
}

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
        expression: &'cache ProcessedFormatString<'static>,
        append_newline: bool,
        decoded_values: DecodedValues<'cache>,
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
