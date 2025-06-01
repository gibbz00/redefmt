use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatArgument<'a> {
    Index(usize),
    #[cfg_attr(feature = "serde", serde(borrow))]
    Identifier(ArgumentIdentifier<'a>),
}

impl<'a> FormatArgument<'a> {
    /// Context from `FormatSegment::parse`:
    /// - `str` not empty
    pub(crate) fn parse(offset: usize, str: &'a str) -> Result<Self, FormatStringParseError> {
        match str.starts_with(|ch: char| ch.is_ascii_digit()) {
            true => Integer::parse(offset, str).map(|integer| FormatArgument::Index(integer.inner())),
            false => ArgumentIdentifier::parse_impl(offset, str).map(FormatArgument::Identifier),
        }
    }

    pub(crate) fn owned(&self) -> FormatArgument<'static> {
        match self {
            FormatArgument::Index(integer) => FormatArgument::Index(*integer),
            FormatArgument::Identifier(identifier) => FormatArgument::Identifier(identifier.owned()),
        }
    }

    #[cfg(feature = "syn")]
    pub(crate) fn matches_index(&self, index: usize) -> bool {
        match self {
            FormatArgument::Index(argument_index) => *argument_index == index,
            FormatArgument::Identifier(_) => false,
        }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for FormatArgument<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let variant_tokens = match self {
            FormatArgument::Index(index) => quote! { Index(#index) },
            FormatArgument::Identifier(argument_identifier) => quote! { Identifier(#argument_identifier) },
        };

        let argument_tokens = quote! {
            ::redefmt_args::format_string::argument::FormatArgument::#variant_tokens
        };

        tokens.extend(argument_tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_index() {
        let str = "01";

        let expected_argument = {
            let integer = Integer::parse(0, str).unwrap();
            FormatArgument::Index(integer.inner())
        };

        let actual_argument = FormatArgument::parse(0, str).unwrap();

        assert_eq!(expected_argument, actual_argument);
    }

    #[test]
    fn parse_identifier() {
        let str = "x";

        let expected_argument = {
            let identifier = ArgumentIdentifier::parse(str).unwrap();
            FormatArgument::Identifier(identifier)
        };

        let actual_argument = FormatArgument::parse(0, str).unwrap();

        assert_eq!(expected_argument, actual_argument);
    }
}
