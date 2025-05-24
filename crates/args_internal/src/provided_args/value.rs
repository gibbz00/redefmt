use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProvidedArgValue<'a> {
    Literal(ProvidedArgLiteral),
    Variable(#[cfg_attr(feature = "serde", serde(borrow))] AnyIdentifier<'a>),
}

impl ProvidedArgValue<'_> {
    pub(crate) fn is_dynamic(&self) -> bool {
        match self {
            ProvidedArgValue::Literal(_) => false,
            ProvidedArgValue::Variable(_) => true,
        }
    }
}

#[cfg(feature = "syn")]
impl syn::parse::Parse for ProvidedArgValue<'static> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(syn::Lit) {
            input.parse().map(ProvidedArgValue::Literal)
        } else if lookahead.peek(syn::Ident) {
            input.parse().map(ProvidedArgValue::Variable)
        } else {
            Err(lookahead.error())
        }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for ProvidedArgValue<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let variant_tokens = match self {
            ProvidedArgValue::Literal(provided_arg_literal) => quote! { Literal(#provided_arg_literal) },
            ProvidedArgValue::Variable(any_identifier) => quote! { Variable(#any_identifier) },
        };

        let provided_arg_value_tokens = quote! {
            ::redefmt_args::provided_args::ProvidedArgValue::#variant_tokens
        };

        tokens.extend(provided_arg_value_tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn parse_literal() {
        let actual = parse_quote!("test");
        let expected = ProvidedArgValue::Literal(parse_quote!("test"));
        assert_eq!(expected, actual);

        let actual = parse_quote!(10);
        let expected = ProvidedArgValue::Literal(parse_quote!(10));
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_variable() {
        let actual = parse_quote!(x);
        let expected = ProvidedArgValue::Variable(parse_quote!(x));
        assert_eq!(expected, actual);
    }
}
