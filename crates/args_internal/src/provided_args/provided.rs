use alloc::vec::Vec;

use hashbrown::HashSet;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
#[impl_tools::autoimpl(Default)]
pub struct ProvidedArgs<'a, E> {
    pub positional: Vec<E>,
    pub named: Vec<(AnyIdentifier<'a>, E)>,
}

impl<E> ProvidedArgs<'_, E> {
    pub fn expressions(&self) -> impl Iterator<Item = &E> {
        let positional_iter = self.positional.iter();
        let named_iter = self.named.iter().map(|(_, expr)| expr);

        positional_iter.chain(named_iter)
    }
}

impl syn::parse::Parse for ProvidedArgs<'static, syn::Expr> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut positional_args = Vec::new();
        let mut named = Vec::new();
        let mut registered_named_args = HashSet::new();

        loop {
            if input.is_empty() {
                break;
            }

            // named before positional so to not parse `x` in `x = 10` as a positional argument
            if input.peek2(syn::Token![=]) {
                let name: AnyIdentifier = input.parse()?;
                let eq_token = input.parse::<syn::Token![=]>()?;
                let value = input.parse()?;

                // unraw because `x = 10, r#x = 20` should count as duplicate identifiers
                let unrawed_name = name.clone().unraw();

                if registered_named_args.contains(&unrawed_name) {
                    // IMPROVEMENT: span points to `name`
                    return Err(syn::Error::new(eq_token.span, "duplicate argument names not allowed"));
                } else {
                    named.push((name, value));
                    registered_named_args.insert(unrawed_name);
                }
            } else {
                let positional_arg = input.parse()?;

                if !named.is_empty() {
                    return Err(syn::Error::new(
                        // IMPROVEMENT: span points to `positional arg`
                        input.span(),
                        "positional arguments can not follow named arguments",
                    ));
                } else {
                    positional_args.push(positional_arg);
                }
            }

            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self { positional: positional_args, named })
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

    #[test]
    #[should_panic]
    fn duplicate_name_error() {
        let _: ProvidedArgs<_> = parse_quote!(x = 10, x = 20);
    }

    #[test]
    #[should_panic]
    fn duplicate_name_error_when_raw() {
        let _: ProvidedArgs<_> = parse_quote!(x = 10, r#x = 20);
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
