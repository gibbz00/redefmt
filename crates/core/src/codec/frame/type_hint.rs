#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr, strum::Display)]
#[repr(u8)]
pub enum TypeHint {
    // ** Primitives ** 0XX
    Boolean = 0,
    Usize = 10,
    U8 = 11,
    U16 = 12,
    U32 = 13,
    U64 = 14,
    U128 = 15,
    Isize = 20,
    I8 = 21,
    I16 = 22,
    I32 = 23,
    I64 = 24,
    I128 = 25,
    // Gives space for fsize, mini (F8), and half (F16) floating
    // precision types, if introduced later on.
    F32 = 33,
    F64 = 34,

    // ** Collections ** 1XX

    // length + type hint for each value
    // (effectively a dyn slice)
    Tuple = 100,

    // length hint
    Char = 101,
    // length hint
    StringSlice = 102,

    // List and DynList reused between array, vec and slice

    // length + leading type hint if length > 0
    List = 103,
    // length + type hint for each element
    DynList = 104,

    // * Meta * 2XX
    WriteStatements = 201,
    TypeStructure = 202,
}
