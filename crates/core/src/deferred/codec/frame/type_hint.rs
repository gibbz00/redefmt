#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl TypeHint {
    pub const fn from_repr(repr: u8) -> Option<Self> {
        let type_hint = match repr {
            0 => Self::Boolean,
            10 => Self::Usize,
            11 => Self::U8,
            12 => Self::U16,
            13 => Self::U32,
            14 => Self::U64,
            15 => Self::U128,
            20 => Self::Isize,
            21 => Self::I8,
            22 => Self::I16,
            23 => Self::I32,
            24 => Self::I64,
            25 => Self::I128,
            33 => Self::F32,
            34 => Self::F64,
            100 => Self::Tuple,
            101 => Self::Char,
            102 => Self::StringSlice,
            103 => Self::List,
            104 => Self::DynList,
            201 => Self::WriteStatements,
            202 => Self::TypeStructure,
            _ => return None,
        };

        Some(type_hint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repr_bijectivity() {
        assert_repr_bijectivity(TypeHint::Boolean);
        assert_repr_bijectivity(TypeHint::Usize);
        assert_repr_bijectivity(TypeHint::U8);
        assert_repr_bijectivity(TypeHint::U16);
        assert_repr_bijectivity(TypeHint::U32);
        assert_repr_bijectivity(TypeHint::U64);
        assert_repr_bijectivity(TypeHint::U128);
        assert_repr_bijectivity(TypeHint::Isize);
        assert_repr_bijectivity(TypeHint::I8);
        assert_repr_bijectivity(TypeHint::I16);
        assert_repr_bijectivity(TypeHint::I32);
        assert_repr_bijectivity(TypeHint::I64);
        assert_repr_bijectivity(TypeHint::I128);
        assert_repr_bijectivity(TypeHint::F32);
        assert_repr_bijectivity(TypeHint::F64);
        assert_repr_bijectivity(TypeHint::Tuple);
        assert_repr_bijectivity(TypeHint::Char);
        assert_repr_bijectivity(TypeHint::StringSlice);
        assert_repr_bijectivity(TypeHint::List);
        assert_repr_bijectivity(TypeHint::DynList);
        assert_repr_bijectivity(TypeHint::WriteStatements);
        assert_repr_bijectivity(TypeHint::TypeStructure);

        fn assert_repr_bijectivity(type_hint: TypeHint) {
            let repr = type_hint as u8;
            let from_repr = TypeHint::from_repr(repr).unwrap();
            assert_eq!(type_hint, from_repr);
        }
    }
}
