/// <https://doc.rust-lang.org/std/fmt/index.html#sign0>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Sign {
    /// '+'
    Plus,
    /// '-'
    Minus,
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Sign {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let variant_name = match self {
            Sign::Plus => quote! { Plus },
            Sign::Minus => quote! { Minus },
        };

        let sign_tokens = quote! {
            ::redefmt_args::format_options::Sign::#variant_name
        };

        tokens.extend(sign_tokens);
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;

    #[test]
    fn to_tokens_plus() {
        let input = Sign::Plus;
        let expected = quote! { ::redefmt_args::format_options::Sign::Plus };

        crate::quote_utils::assert_tokens(input, expected);
    }

    #[test]
    fn to_tokens_minus() {
        let input = Sign::Minus;
        let expected = quote! { ::redefmt_args::format_options::Sign::Minus };

        crate::quote_utils::assert_tokens(input, expected);
    }
}
