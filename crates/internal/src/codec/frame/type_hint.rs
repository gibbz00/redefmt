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

    // TODO:
    // length + leading type hint
    // (no DynSet since Hash is not dyn compatible)
    // Set = 105,
    // length + two leading type hints
    // Map = 107,
    // length + leading type hint + type hint for each element
    // (no need for DynDynMap since Hash is not dyn compatible)
    // DynMap = 108,

    // * Meta * 2XX
    //
    // (Current XX = 13 repr implies u32 write_id, (u16 crate_id + u16 write_statement_id))
    WriteId = 213,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_content_id_type_hint() {
        let write_content_section = 200;
        let write_id_type_hint_repr = TypeHint::WriteId as u8;
        let u32_type_hint_repr = TypeHint::U32 as u8;

        assert_eq!(u32_type_hint_repr, write_id_type_hint_repr - write_content_section)
    }
}
