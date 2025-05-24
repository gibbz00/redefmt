use alloc::vec::Vec;

use hashbrown::HashSet;

use crate::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProvidedArgs<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub positional: Vec<ProvidedArgValue<'a>>,
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub named: ProvidedNamedArgsMap<'a>,
}

impl<'a> ProvidedArgs<'a> {
    /// Count the number of dynamic arguments in positional and named
    ///
    /// Static arguments would in this case be literals, and dynamic those bound
    /// from variables in scope.
    pub fn dynamic_count(&self) -> usize {
        let positional_iter = self.positional.iter();
        let named_iter = self.named.iter().map(|(_, arg)| arg);

        positional_iter
            .chain(named_iter)
            .fold(0, |count, arg| if arg.is_dynamic() { count + 1 } else { count })
    }

    pub fn dynamic_args(&self) -> impl Iterator<Item = &AnyIdentifier<'a>> {
        let positional_iter = self.positional.iter();
        let named_iter = self.named.iter().map(|(_, arg)| arg);

        positional_iter.chain(named_iter).filter_map(|arg| match arg {
            ProvidedArgValue::Literal(_) => None,
            ProvidedArgValue::Variable(identifier) => Some(identifier),
        })
    }

    pub(crate) fn collect_named_set(&self) -> HashSet<ArgumentIdentifier<'static>> {
        self.named
            .iter()
            // idents unrawed since checks with format string arguments need to
            // match against both raw and non-raw alternatives
            .map(|(ident, _)| ident.clone().unraw().owned())
            .collect::<HashSet<_>>()
    }
}

#[cfg(feature = "syn")]
impl syn::parse::Parse for ProvidedArgs<'static> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut positional_args = Vec::new();
        let mut named_args = Vec::new();
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

                if registered_named_args.contains(&name) {
                    // IMPROVEMENT: span points to `name`
                    return Err(syn::Error::new(eq_token.span, "duplicate argument names not allowed"));
                } else {
                    named_args.push((name.clone(), value));
                    registered_named_args.insert(name);
                }
            } else {
                let positional_arg: ProvidedArgValue = input.parse()?;

                if !named_args.is_empty() {
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

        Ok(Self {
            positional: positional_args,
            named: ProvidedNamedArgsMap::new(named_args),
        })
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for ProvidedArgs<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let positional_elements = self.positional.iter();
        let named = &self.named;

        let provided_args_tokens = quote! {
            ::redefmt_args::provided_args::ProvidedArgs {
                positional: [#(#positional_elements),*].into_iter().collect(),
                named: #named,
            }
        };

        tokens.extend(provided_args_tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn dynamic_count() {
        let args: ProvidedArgs = parse_quote!(1, y = false);
        assert_eq!(0, args.dynamic_count());

        let args: ProvidedArgs = parse_quote!(x, y = y);
        assert_eq!(2, args.dynamic_count());
    }

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
        let _: ProvidedArgs = parse_quote!(x = 10, "x");
    }

    #[test]
    #[should_panic]
    fn duplicate_name_error() {
        let _: ProvidedArgs = parse_quote!(x = 10, x = 20);
    }

    fn mock_positional() -> Vec<ProvidedArgValue<'static>> {
        alloc::vec![parse_quote!(10), parse_quote!(a), parse_quote!("y")]
    }

    fn mock_named() -> ProvidedNamedArgsMap<'static> {
        let inner = [
            (AnyIdentifier::parse("x").unwrap(), parse_quote!(10)),
            (AnyIdentifier::parse("match").unwrap(), parse_quote!(y)),
        ]
        .into_iter()
        .collect();

        ProvidedNamedArgsMap::new(inner)
    }
}
