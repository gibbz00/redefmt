use crate::*;

/// <https://doc.rust-lang.org/std/fmt/index.html#formatting-traits>
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatTrait {
    /// ''
    #[default]
    Display,
    /// '?'
    Debug,
    /// 'x?'
    DebugLowerHex,
    /// 'X?'
    DebugUpperHex,
    /// 'o'
    Octal,
    /// 'x'
    LowerHex,
    /// 'X'
    UpperHex,
    /// 'p'
    Pointer,
    /// 'b'
    Binary,
    /// 'e'
    LowerExp,
    /// 'E'
    UpperExp,
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum FormatTraitParseError {
    #[error("unknown variant")]
    Unknown,
}

impl FormatTrait {
    /// Context from `FormatOptions::parse`:
    /// - `str` not empty
    pub(crate) fn parse(offset: usize, str: &str) -> Result<Self, FormatStringParseError> {
        let format_trait = match str {
            "?" => FormatTrait::Debug,
            "x?" => FormatTrait::DebugLowerHex,
            "X?" => FormatTrait::DebugUpperHex,
            "o" => FormatTrait::Octal,
            "x" => FormatTrait::LowerHex,
            "X" => FormatTrait::UpperHex,
            "p" => FormatTrait::Pointer,
            "b" => FormatTrait::Binary,
            "e" => FormatTrait::LowerExp,
            "E" => FormatTrait::UpperExp,
            _ => {
                return Err(FormatStringParseError::new(
                    offset,
                    0..str.len(),
                    FormatTraitParseError::Unknown,
                ));
            }
        };

        Ok(format_trait)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for FormatTrait {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let variant_tokens = match self {
            FormatTrait::Display => quote! { Display },
            FormatTrait::Debug => quote! { Debug },
            FormatTrait::DebugLowerHex => quote! { DebugLowerHex },
            FormatTrait::DebugUpperHex => quote! { DebugUpperHex },
            FormatTrait::Octal => quote! { Octal },
            FormatTrait::LowerHex => quote! { LowerHex },
            FormatTrait::UpperHex => quote! { UpperHex },
            FormatTrait::Pointer => quote! { Pointer },
            FormatTrait::Binary => quote! { Binary },
            FormatTrait::LowerExp => quote! { LowerExp },
            FormatTrait::UpperExp => quote! { UpperExp },
        };

        let trait_tokens = quote! {
            ::redefmt_args::format_string::options::FormatTrait::#variant_tokens
        };

        tokens.extend(trait_tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(FormatTrait::Display, Default::default())
    }

    #[test]
    fn parse() {
        assert_parse("?", FormatTrait::Debug);
        assert_parse("x?", FormatTrait::DebugLowerHex);
        assert_parse("X?", FormatTrait::DebugUpperHex);
        assert_parse("o", FormatTrait::Octal);
        assert_parse("x", FormatTrait::LowerHex);
        assert_parse("X", FormatTrait::UpperHex);
        assert_parse("p", FormatTrait::Pointer);
        assert_parse("b", FormatTrait::Binary);
        assert_parse("e", FormatTrait::LowerExp);
        assert_parse("E", FormatTrait::UpperExp);
    }

    #[test]
    fn parse_error() {
        let expected_error = FormatStringParseError::new(0, 0..2, FormatTraitParseError::Unknown);

        let actual_error = FormatTrait::parse(0, "xx").unwrap_err();

        assert_eq!(expected_error, actual_error);
    }

    fn assert_parse(str: &str, expected: FormatTrait) {
        let actual = FormatTrait::parse(0, str).unwrap();
        assert_eq!(expected, actual);
    }
}
