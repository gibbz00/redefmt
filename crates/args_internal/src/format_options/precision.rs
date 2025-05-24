use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum FormatPrecisionParseError {
    #[error("empty string after '.', expected a precision count or argument")]
    Empty,
}

/// <https://doc.rust-lang.org/std/fmt/index.html#precision>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatPrecision<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    Count(FormatCount<'a>),
    /// '*'
    NextArgument,
}

impl FormatPrecision<'_> {
    pub(crate) fn owned(&self) -> FormatPrecision<'static> {
        match self {
            FormatPrecision::Count(count) => FormatPrecision::Count(count.owned()),
            FormatPrecision::NextArgument => FormatPrecision::NextArgument,
        }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for FormatPrecision<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let variant_tokens = match self {
            FormatPrecision::Count(format_count) => quote! { Count(#format_count) },
            FormatPrecision::NextArgument => quote! { NextArgument },
        };

        let precision_tokens = quote! {
            ::redefmt_args::format_options::FormatPrecision::#variant_tokens
        };

        tokens.extend(precision_tokens);
    }
}
