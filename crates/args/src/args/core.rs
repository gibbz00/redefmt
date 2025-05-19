use alloc::vec::Vec;

use hashbrown::HashMap;
use syn::{Token, parse::Parse};

use crate::*;

#[derive(Debug, PartialEq)]
pub struct Args {
    pub positional: Vec<ArgValue>,
    // important to parse with simply `syn::Ident` since names are `IDENTIFIER_OR_KEYWORD`
    pub named: HashMap<Identifier<'static>, ArgValue>,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut positional_args = Vec::new();
        let mut named_args = HashMap::new();

        loop {
            if input.is_empty() {
                break;
            }

            // named before positional so to not parse `x` in `x = 10` as a positional argument
            if input.peek2(Token![=]) {
                let name = input.parse::<ArgName>()?;
                let _ = input.parse::<Token![=]>()?;
                let value = input.parse()?;

                if named_args.insert(name.identifier, value).is_some() {
                    return Err(syn::Error::new(name.span, "duplicate argument names not allowed"));
                }
            } else {
                let positional_arg: ArgValue = input.parse()?;

                if !named_args.is_empty() {
                    return Err(syn::Error::new(
                        positional_arg.span(),
                        "positional arguments can not follow named arguments",
                    ));
                } else {
                    positional_args.push(positional_arg);
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { positional: positional_args, named: named_args })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn parse_empty() {
        let expected = Args { positional: Default::default(), named: Default::default() };

        let actual = parse_quote!();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_positional() {
        let expected = Args { positional: mock_positional(), named: Default::default() };

        let actual = parse_quote!(10, a, "y");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_named() {
        let expected = Args { positional: Default::default(), named: mock_named() };

        let actual = parse_quote!(x = 10, match = y);

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_combined() {
        let expected = Args { positional: mock_positional(), named: mock_named() };

        let actual = parse_quote!(10, a, "y", x = 10, match = y);

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_with_trailing_comma() {
        let expected = Args { positional: Default::default(), named: mock_named() };

        let actual = parse_quote!(x = 10, match = y,);

        assert_eq!(expected, actual);
    }

    #[test]
    #[should_panic]
    fn positional_after_named_error() {
        let _: Args = parse_quote!(x = 10, "x");
    }

    #[test]
    #[should_panic]
    fn duplicate_name_error() {
        let _: Args = parse_quote!(x = 10, x = 20);
    }

    fn mock_positional() -> Vec<ArgValue> {
        alloc::vec![parse_quote!(10), parse_quote!(a), parse_quote!("y")]
    }

    fn mock_named() -> HashMap<Identifier<'static>, ArgValue> {
        HashMap::from_iter([
            (Identifier::parse(0, "x").unwrap(), parse_quote!(10)),
            (Identifier::parse(0, "match").unwrap(), parse_quote!(y)),
        ])
    }
}
