use alloc::vec::Vec;

use hashbrown::HashSet;

use crate::*;

pub trait ProvidedArgExpression {
    fn from_identifier(any_identifier: AnyIdentifier<'_>) -> Self;

    fn unmove_expression(&mut self);
}

#[derive(Debug, Clone, PartialEq)]
#[impl_tools::autoimpl(Default)]
pub struct ProvidedArgs<'a, E> {
    pub(crate) positional: Vec<E>,
    pub(crate) named: Vec<(AnyIdentifier<'a>, E)>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProvidedArgsError {
    #[error("duplicate argument names not allowed, namely: {0}")]
    ProvidedDuplicate(ArgumentIdentifier<'static>),
}

impl<'a, E> ProvidedArgs<'a, E> {
    /// Verifies that all named arguments are unique in their unrawed form
    ///
    /// `x = 10, r#x = 20` counts in other words as a duplicate match.
    pub fn new(positional: Vec<E>, named: Vec<(AnyIdentifier<'a>, E)>) -> Result<Self, ProvidedArgsError> {
        let mut registered_named_args = HashSet::new();

        for (name, _) in &named {
            let unrawed_name = name.clone().unraw();

            if registered_named_args.contains(&unrawed_name) {
                return Err(ProvidedArgsError::ProvidedDuplicate(unrawed_name.owned()));
            } else {
                registered_named_args.insert(unrawed_name);
            }
        }

        Ok(Self { positional, named })
    }

    pub fn dissolve_args(self) -> (Vec<E>, Vec<(AnyIdentifier<'a>, E)>) {
        (self.positional, self.named)
    }

    pub fn dissolve_expressions(self) -> (Vec<E>, Vec<AnyIdentifier<'a>>) {
        let mut expressions = self.positional.into_iter().collect::<Vec<_>>();

        let identifiers = self
            .named
            .into_iter()
            .map(|(identifier, expr)| {
                expressions.push(expr);
                identifier
            })
            .collect();

        (expressions, identifiers)
    }
}

#[cfg(feature = "syn")]
mod syn_impl {
    use super::*;

    impl ProvidedArgExpression for syn::Expr {
        fn from_identifier(any_identifier: AnyIdentifier<'_>) -> Self {
            from_identifier_impl(any_identifier.into())
        }

        fn unmove_expression(&mut self) {
            if let syn::Expr::Path(path) = self
                && let Some(ident) = path.path.get_ident().cloned()
            {
                *self = from_identifier_impl(ident);
            }
        }
    }

    fn from_identifier_impl(ident: syn::Ident) -> syn::Expr {
        syn::Expr::Reference(syn::ExprReference {
            attrs: Default::default(),
            and_token: Default::default(),
            mutability: None,
            expr: alloc::boxed::Box::new(syn::Expr::Path(syn::ExprPath {
                attrs: Default::default(),
                qself: None,
                path: syn::Path::from(ident),
            })),
        })
    }

    impl syn::parse::Parse for ProvidedArgs<'static, syn::Expr> {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            use alloc::string::ToString;

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

            // IMPROVEMENT: span points to argument that emitted error?
            Self::new(positional, named).map_err(|err| syn::Error::new(input.span(), err.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    // constructor tests

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

    // syn parse tests

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
