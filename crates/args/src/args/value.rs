use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};

#[derive(Debug, PartialEq)]
pub enum ArgValue {
    Literal(syn::Lit),
    Variable(syn::Ident),
}

impl ArgValue {
    pub fn span(&self) -> Span {
        match self {
            ArgValue::Literal(lit) => lit.span(),
            ArgValue::Variable(ident) => ident.span(),
        }
    }
}

impl Parse for ArgValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(syn::Lit) {
            input.parse().map(ArgValue::Literal)
        } else if lookahead.peek(syn::Ident) {
            input.parse().map(ArgValue::Variable)
        } else {
            Err(lookahead.error())
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
        let expected = ArgValue::Literal(parse_quote!("test"));
        assert_eq!(expected, actual);

        let actual = parse_quote!(10);
        let expected = ArgValue::Literal(parse_quote!(10));
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_variable() {
        let actual = parse_quote!(x);
        let expected = ArgValue::Variable(parse_quote!(x));
        assert_eq!(expected, actual);
    }
}
