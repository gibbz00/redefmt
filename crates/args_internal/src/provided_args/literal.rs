use alloc::{ffi::CString, string::String, vec::Vec};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProvidedLiteral {
    Str(String),
    ByteStr(Vec<u8>),
    CStr(CString),
    Byte(u8),
    Char(char),
    /// base 10 representatino + suffix (which may be empty)
    Int(String, String),
    /// base 10 representatino + suffix (which may be empty)
    Float(String, String),
    Bool(bool),
}

#[cfg(feature = "syn")]
mod syn {
    use alloc::string::ToString;

    use super::*;

    impl ProvidedLiteral {
        fn from_syn(syn_lit: ::syn::Lit) -> Self {
            match syn_lit {
                ::syn::Lit::Str(lit) => Self::Str(lit.value()),
                ::syn::Lit::ByteStr(lit) => Self::ByteStr(lit.value()),
                ::syn::Lit::CStr(lit) => Self::CStr(lit.value()),
                ::syn::Lit::Byte(lit) => Self::Byte(lit.value()),
                ::syn::Lit::Char(lit) => Self::Char(lit.value()),
                ::syn::Lit::Int(lit) => Self::Int(lit.base10_digits().to_string(), lit.suffix().to_string()),
                ::syn::Lit::Float(lit) => Self::Float(lit.base10_digits().to_string(), lit.suffix().to_string()),
                ::syn::Lit::Bool(lit) => Self::Bool(lit.value()),
                ::syn::Lit::Verbatim(_) => unreachable!("literal not interpreted by syn"),
                // ::syn::Lit is non-exhaustive
                _ => unimplemented!(),
            }
        }
    }

    impl ::syn::parse::Parse for ProvidedLiteral {
        fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
            input.parse().map(Self::from_syn)
        }
    }
}
