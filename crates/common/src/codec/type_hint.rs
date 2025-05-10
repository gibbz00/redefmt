#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr, strum::Display)]
#[repr(u8)]
pub enum TypeHint {
    // ** Primitives ** 0XX
    Boolean = 0,
    U8 = 10,
    U16 = 11,
    U32 = 12,
    U64 = 13,
    U128 = 14,
    I8 = 20,
    I16 = 21,
    I32 = 22,
    I64 = 23,
    // Gives space for mini (F8) and half (F16) floating
    // precision types, if introduced later on.
    F32 = 32,
    F64 = 33,

    // * Write Content ID * 1XX
    //
    // (Current XX = 12 repr implies u32 write content ID)
    WriteContentId = 112,

    // ** Collections ** 2XX

    // length hint
    StringSlice = 200,

    // length + type hint for each value
    // (effectively a dyn slice)
    Tuple = 201,

    // length + leading type hint
    Slice = 202,
    // length + type hint for each element
    DynSlice = 203,

    // length + leading type hint
    // (no DynSet since Hash is not dyn compatible)
    Set = 204,

    // length + two leading type hints
    Map = 206,
    // length + leading type hint + type hint for each element
    // (no need for DynDynMap since Hash is not dyn compatible)
    DynMap = 207,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_content_id_type_hint() {
        let write_content_section = 100;
        let write_content_id_type_hint_repr = TypeHint::WriteContentId as u8;
        let u32_type_hint_repr = TypeHint::U32 as u8;

        assert_eq!(
            u32_type_hint_repr,
            write_content_id_type_hint_repr - write_content_section
        )
    }
}
