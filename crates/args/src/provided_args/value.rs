use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProvidedArgValue {
    Literal(syn::Lit),
    Variable(syn::Ident),
}

impl ProvidedArgValue {
    pub fn span(&self) -> Span {
        match self {
            ProvidedArgValue::Literal(lit) => lit.span(),
            ProvidedArgValue::Variable(ident) => ident.span(),
        }
    }
}

impl Parse for ProvidedArgValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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

#[cfg(feature = "provided-args-serde")]
mod serde {
    use ::serde::{Deserialize, Serialize};
    use syn_serde::Syn;

    use super::*;

    #[derive(Serialize, Deserialize)]
    enum ProvidedArgValueAdapter {
        Literal(<syn::Lit as syn_serde::Syn>::Adapter),
        Variable(<syn::Ident as syn_serde::Syn>::Adapter),
    }

    impl ProvidedArgValueAdapter {
        fn to_adapted(&self) -> ProvidedArgValue {
            match self {
                ProvidedArgValueAdapter::Literal(adapter) => ProvidedArgValue::Literal(Syn::from_adapter(adapter)),
                ProvidedArgValueAdapter::Variable(adapter) => ProvidedArgValue::Variable(Syn::from_adapter(adapter)),
            }
        }

        fn from_adapted(arg_value: &ProvidedArgValue) -> Self {
            match arg_value {
                ProvidedArgValue::Literal(lit) => ProvidedArgValueAdapter::Literal(lit.to_adapter()),
                ProvidedArgValue::Variable(ident) => ProvidedArgValueAdapter::Variable(ident.to_adapter()),
            }
        }
    }

    impl Serialize for ProvidedArgValue {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            ProvidedArgValueAdapter::from_adapted(self).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for ProvidedArgValue {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ProvidedArgValueAdapter::deserialize(deserializer).map(|adapter| adapter.to_adapted())
        }
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
