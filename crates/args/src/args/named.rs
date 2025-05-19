use syn::{
    Token,
    parse::{Parse, ParseStream},
};

use crate::*;

#[derive(Debug, PartialEq)]
pub struct NamedArg {
    // important to parse with simply `syn::Ident` since names are `IDENTIFIER_OR_KEYWORD`
    pub name: Identifier<'static>,
    pub value: ArgValue,
}

impl Parse for NamedArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse()?;

        Ok(Self { name, value })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn parse() {
        let actual = parse_quote!(match = x);

        let expected = NamedArg {
            name: Identifier::parse(0, "match").unwrap(),
            value: ArgValue::Variable(syn::parse_str("x").unwrap()),
        };

        assert_eq!(expected, actual);
    }
}
