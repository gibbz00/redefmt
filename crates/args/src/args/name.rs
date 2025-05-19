use alloc::string::ToString;

use proc_macro2::Span;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
};

use crate::*;

pub struct ArgName {
    // important to parse with simply `syn::Ident` since names are `IDENTIFIER_OR_KEYWORD`
    pub identifier: Identifier<'static>,
    pub span: Span,
}

impl Parse for ArgName {
    fn parse(input: ParseStream) -> ::syn::Result<Self> {
        let ident = input.call(::syn::Ident::parse_any)?;

        // HACK:
        // proc_macro2 exposes neither `is_raw` nor `sym`
        // so we can't just `Identifier { inner: ident.sym(), raw: ident.is_raw() }`
        let identifier = Identifier::parse(0, ident.to_string())
            .expect("Identifier::parse failed when ::syn::Ident::parse_any succeeded");

        Ok(Self { identifier, span: ident.span() })
    }
}

#[cfg(test)]
mod tests {
    use ::syn::parse_quote;

    use super::*;

    #[test]
    fn parse_keyword() {
        let actual: ArgName = parse_quote!(match);
        let expected = Identifier { raw: false, inner: "match".into() };

        assert_eq!(expected, actual.identifier);
    }

    #[test]
    fn parse_ident() {
        let actual: ArgName = parse_quote!(test);
        let expected = Identifier { raw: false, inner: "test".into() };

        assert_eq!(expected, actual.identifier);
    }

    #[test]
    fn parse_raw_ident() {
        let actual: ArgName = parse_quote!(r#test);
        let expected = Identifier { raw: true, inner: "test".into() };

        assert_eq!(expected, actual.identifier);
    }
}
