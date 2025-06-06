use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum FormatCountParseError {
    #[error("no closing '$' found")]
    UnclosedArgument,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatCount<'a> {
    Integer(usize),
    #[cfg_attr(feature = "serde", serde(borrow))]
    Argument(FormatArgument<'a>),
}

impl FormatCount<'_> {
    pub(crate) fn owned(&self) -> FormatCount<'static> {
        match self {
            FormatCount::Integer(integer) => FormatCount::Integer(*integer),
            FormatCount::Argument(argument) => FormatCount::Argument(argument.owned()),
        }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for FormatCount<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let variant_tokens = match self {
            FormatCount::Integer(integer) => quote! { Integer(#integer) },
            FormatCount::Argument(format_argument) => quote! { Argument(#format_argument) },
        };

        let count_tokens = quote! {
            ::redefmt_args::format_string::options::FormatCount::#variant_tokens
        };

        tokens.extend(count_tokens);
    }
}
