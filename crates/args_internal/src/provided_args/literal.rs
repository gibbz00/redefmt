use alloc::{borrow::Cow, ffi::CString, vec::Vec};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProvidedArgLiteral<'a> {
    Str(Cow<'a, str>),
    ByteStr(Vec<u8>),
    CStr(CString),
    Byte(u8),
    Char(char),
    Boolean(bool),

    // Numbers
    Usize(u64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    UnknownUint(u128),
    Isize(i64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    UnknownInt(i128),
    /// Big Endian
    F32([u8; 4]),
    /// Big Endian
    F64([u8; 8]),
    /// Big Endian
    UnknownFloat([u8; 8]),
}

#[cfg(feature = "syn")]
mod syn {
    use ::syn::{LitFloat, LitInt};

    use super::*;

    impl ::syn::parse::Parse for ProvidedArgLiteral<'static> {
        fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
            let provided_literal = match input.parse()? {
                ::syn::Lit::Str(lit) => Self::Str(lit.value().into()),
                ::syn::Lit::ByteStr(lit) => Self::ByteStr(lit.value()),
                ::syn::Lit::CStr(lit) => Self::CStr(lit.value()),
                ::syn::Lit::Byte(lit) => Self::Byte(lit.value()),
                ::syn::Lit::Char(lit) => Self::Char(lit.value()),
                ::syn::Lit::Int(lit) => parse_lit_int(lit),
                ::syn::Lit::Float(lit) => parse_lit_float(lit),
                ::syn::Lit::Bool(lit) => Self::Boolean(lit.value()),
                ::syn::Lit::Verbatim(_) => unreachable!("literal not interpreted by syn"),
                // ::syn::Lit is non-exhaustive
                _ => unimplemented!(),
            };

            Ok(provided_literal)
        }
    }

    fn parse_lit_int(lit_int: LitInt) -> ProvidedArgLiteral<'static> {
        let result = match lit_int.suffix() {
            "usize" => lit_int.base10_parse().map(ProvidedArgLiteral::Usize),
            "u8" => lit_int.base10_parse().map(ProvidedArgLiteral::U8),
            "u16" => lit_int.base10_parse().map(ProvidedArgLiteral::U16),
            "u32" => lit_int.base10_parse().map(ProvidedArgLiteral::U32),
            "u64" => lit_int.base10_parse().map(ProvidedArgLiteral::U64),
            "u128" => lit_int.base10_parse().map(ProvidedArgLiteral::U128),
            "isize" => lit_int.base10_parse().map(ProvidedArgLiteral::Isize),
            "i8" => lit_int.base10_parse().map(ProvidedArgLiteral::I8),
            "i16" => lit_int.base10_parse().map(ProvidedArgLiteral::I16),
            "i32" => lit_int.base10_parse().map(ProvidedArgLiteral::I32),
            "i64" => lit_int.base10_parse().map(ProvidedArgLiteral::I64),
            "i128" => lit_int.base10_parse().map(ProvidedArgLiteral::I128),
            _ => match lit_int.base10_digits().starts_with('-') {
                true => lit_int.base10_parse().map(ProvidedArgLiteral::UnknownInt),
                false => lit_int.base10_parse().map(ProvidedArgLiteral::UnknownUint),
            },
        };

        result.expect("parsed syn::LitInt not parseable into an integer")
    }

    fn parse_lit_float(lit_float: LitFloat) -> ProvidedArgLiteral<'static> {
        let result = match lit_float.suffix() {
            "f32" => lit_float
                .base10_parse()
                .map(f32::to_be_bytes)
                .map(ProvidedArgLiteral::F32),
            "f64" => lit_float
                .base10_parse()
                .map(f64::to_be_bytes)
                .map(ProvidedArgLiteral::F64),
            "f128" => unimplemented!("unstable `f128` is currently not implemented"),
            _ => lit_float
                .base10_parse()
                .map(f64::to_be_bytes)
                .map(ProvidedArgLiteral::UnknownFloat),
        };

        result.expect("parsed syn::LitFloat not parseable into a float")
    }

    #[cfg(test)]
    mod tests {
        use ::syn::parse_quote;
        use num_traits::ToBytes;

        use super::*;

        #[test]
        fn parse_float() {
            assert_float(1.0f32, ProvidedArgLiteral::F32, parse_quote!(1.0f32));
            assert_float(1.0, ProvidedArgLiteral::F64, parse_quote!(1.0f64));
            assert_float(1.0, ProvidedArgLiteral::UnknownFloat, parse_quote!(1.0));

            fn assert_float<'a, T: ToBytes>(
                inner: T,
                into_expected: impl FnOnce(<T as ToBytes>::Bytes) -> ProvidedArgLiteral<'a>,
                actual: ProvidedArgLiteral,
            ) {
                let inner_bytes = inner.to_be_bytes();
                let expected = into_expected(inner_bytes);
                assert_eq!(expected, actual);
            }
        }

        #[test]
        fn parse_int() {
            assert_uint(ProvidedArgLiteral::U8, parse_quote!(1u8));
            assert_uint(ProvidedArgLiteral::UnknownUint, parse_quote!(1));
            fn assert_uint<'a, T: num_traits::One>(
                into_expected: impl FnOnce(T) -> ProvidedArgLiteral<'a>,
                actual: ProvidedArgLiteral,
            ) {
                let expected = into_expected(T::one());
                assert_eq!(expected, actual);
            }
        }

        #[test]
        fn parse_uint() {
            assert_int(false, ProvidedArgLiteral::I8, parse_quote!(1i8));
            assert_int(true, ProvidedArgLiteral::I8, parse_quote!(-1i8));
            assert_int(true, ProvidedArgLiteral::UnknownInt, parse_quote!(-1));

            fn assert_int<'a, T: num_traits::One + num_traits::Signed>(
                invert: bool,
                into_expected: impl FnOnce(T) -> ProvidedArgLiteral<'a>,
                actual: ProvidedArgLiteral,
            ) {
                let inner = match invert {
                    true => -T::one(),
                    false => T::one(),
                };

                let expected = into_expected(inner);
                assert_eq!(expected, actual);
            }
        }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for ProvidedArgLiteral<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let variant_tokens = match self {
            ProvidedArgLiteral::Str(lit) => quote! { Str(#lit.into()) },
            ProvidedArgLiteral::ByteStr(lit) => quote! { ByteStr([#(#lit),*].into_iter().collect()) },
            ProvidedArgLiteral::CStr(lit) => quote! { CStr(#lit) },
            ProvidedArgLiteral::Byte(lit) => quote! { Byte(#lit) },
            ProvidedArgLiteral::Char(lit) => quote! { Char(#lit) },
            ProvidedArgLiteral::Boolean(lit) => quote! { Boolean(#lit) },
            ProvidedArgLiteral::Usize(lit) => quote! { Usize(#lit) },
            ProvidedArgLiteral::U8(lit) => quote! { U8(#lit) },
            ProvidedArgLiteral::U16(lit) => quote! { U16(#lit) },
            ProvidedArgLiteral::U32(lit) => quote! { U32(#lit) },
            ProvidedArgLiteral::U64(lit) => quote! { U64(#lit) },
            ProvidedArgLiteral::U128(lit) => quote! { U128(#lit) },
            ProvidedArgLiteral::UnknownUint(lit) => quote! { UnknownUint(#lit) },
            ProvidedArgLiteral::Isize(lit) => quote! { Isize(#lit) },
            ProvidedArgLiteral::I8(lit) => quote! { I8(#lit) },
            ProvidedArgLiteral::I16(lit) => quote! { I16(#lit) },
            ProvidedArgLiteral::I32(lit) => quote! { I32(#lit) },
            ProvidedArgLiteral::I64(lit) => quote! { I64(#lit) },
            ProvidedArgLiteral::I128(lit) => quote! { I128(#lit) },
            ProvidedArgLiteral::UnknownInt(lit) => quote! { UnknownInt(#lit) },
            ProvidedArgLiteral::F32(lit) => quote! { F32([#(#lit),*]) },
            ProvidedArgLiteral::F64(lit) => quote! { F64([#(#lit),*]) },
            ProvidedArgLiteral::UnknownFloat(lit) => quote! { UnknownFloat([#(#lit),*]) },
        };

        let provided_literal_tokens = quote! {
            ::redefmt_args::provided_args::ProvidedArgLiteral::#variant_tokens
        };

        tokens.extend(provided_literal_tokens);
    }
}
