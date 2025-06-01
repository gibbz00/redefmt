use alloc::vec::Vec;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
#[impl_tools::autoimpl(Default)]
pub struct ProvidedArgs<'a, E> {
    pub positional: Vec<E>,
    pub named: Vec<(AnyIdentifier<'a>, E)>,
}

#[cfg(feature = "syn")]
impl syn::parse::Parse for ProvidedArgs<'static, syn::Expr> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut positional = Vec::new();
        let mut named = Vec::new();

        loop {
            if input.is_empty() {
                break;
            }

            // named before positional so to not parse `x` in `x = 10` as a positional argument
            if input.peek2(syn::Token![=]) {
                let name: AnyIdentifier = input.parse()?;
                let _ = input.parse::<syn::Token![=]>()?;
                let value = input.parse()?;

                named.push((name, value));
            } else {
                let positional_arg = input.parse()?;

                if !named.is_empty() {
                    return Err(syn::Error::new(
                        input.span(),
                        "positional arguments can not follow named arguments",
                    ));
                } else {
                    positional.push(positional_arg);
                }
            }

            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self { positional, named })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn parse_empty() {
        let expected = ProvidedArgs { positional: Default::default(), named: Default::default() };

        let actual = parse_quote!();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_positional() {
        let expected = ProvidedArgs { positional: mock_positional(), named: Default::default() };

        let actual = parse_quote!(10, a, "y");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_named() {
        let expected = ProvidedArgs { positional: Default::default(), named: mock_named() };

        let actual = parse_quote!(x = 10, match = y);

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_combined() {
        let expected = ProvidedArgs { positional: mock_positional(), named: mock_named() };

        let actual = parse_quote!(10, a, "y", x = 10, match = y);

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_with_trailing_comma() {
        let expected = ProvidedArgs { positional: Default::default(), named: mock_named() };

        let actual = parse_quote!(x = 10, match = y,);

        assert_eq!(expected, actual);
    }

    #[test]
    #[should_panic]
    fn positional_after_named_error() {
        let _: ProvidedArgs<_> = parse_quote!(x = 10, "x");
    }

    fn mock_positional() -> Vec<syn::Expr> {
        alloc::vec![parse_quote!(10), parse_quote!(a), parse_quote!("y")]
    }

    fn mock_named() -> Vec<(AnyIdentifier<'static>, syn::Expr)> {
        [
            (AnyIdentifier::parse("x").unwrap(), parse_quote!(10)),
            (AnyIdentifier::parse("match").unwrap(), parse_quote!(y)),
        ]
        .into_iter()
        .collect()
    }
}
