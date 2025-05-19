use alloc::vec::Vec;

use syn::{Token, parse::Parse};

use crate::*;

#[derive(Debug, PartialEq)]
pub struct Args {
    pub positional: Vec<PositionalArg>,
    pub named: Vec<NamedArg>,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut positional_args = Vec::new();
        let mut named_args = Vec::new();

        loop {
            if input.is_empty() {
                break;
            }

            // named before positional so to not parse `x` in `x = 10` as a positional argument
            if input.peek2(Token![=]) {
                let named_arg = input.parse()?;
                named_args.push(named_arg);
            } else {
                let positional_arg = input.parse()?;

                if !named_args.is_empty() {
                    return Err(syn::Error::new(
                        input.span(),
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
        let expected = Args { positional: Vec::new(), named: Vec::new() };

        let actual = parse_quote!();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_positional() {
        let expected = Args { positional: mock_positional(), named: Vec::new() };

        let actual = parse_quote!(10, a, "y");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_named() {
        let expected = Args { positional: Vec::new(), named: mock_named() };

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
        let expected = Args { positional: Vec::new(), named: mock_named() };

        let actual = parse_quote!(x = 10, match = y,);

        assert_eq!(expected, actual);
    }

    #[test]
    #[should_panic]
    fn posistional_after_named_error() {
        let _: Args = parse_quote!(x = 10, "x");
    }

    fn mock_positional() -> Vec<PositionalArg> {
        let args = [parse_quote!(10), parse_quote!(a), parse_quote!("y")].map(PositionalArg);
        Vec::from_iter(args)
    }

    fn mock_named() -> Vec<NamedArg> {
        alloc::vec![parse_quote!(x = 10), parse_quote!(match = y)]
    }
}
